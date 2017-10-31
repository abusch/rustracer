#[cfg(openexr)]
extern crate openexr;

use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

use img;
#[cfg(openexr)]
use openexr::{FrameBufferMut, InputFile};

use Point2i;
use errors::*;
use spectrum::Spectrum;

pub fn read_image<P: AsRef<Path>>(path: P) -> Result<(Vec<Spectrum>, Point2i)> {
    info!("Loading image {}", path.as_ref().display());
    let path = path.as_ref();
    let extension = path.extension()
        .ok_or("Texture filename doesn't have an extension")?;
    if extension == "tga" || extension == "TGA" || extension == "png" || extension == "PNG" {
        read_image_tga_png(path)
    } else if extension == "exr" || extension == "EXR" {
        read_image_exr(path)
    } else if extension == "pfm" {
        read_image_pfm(path)
    } else {
        bail!("Unsupported file format");
    }
}

fn read_image_tga_png<P: AsRef<Path>>(path: P) -> Result<(Vec<Spectrum>, Point2i)> {
    info!("Loading texture {}", path.as_ref().display());
    let buf = img::open(path)?;

    let rgb = buf.to_rgb();
    let res = Point2i::new(rgb.width() as i32, rgb.height() as i32);
    let pixels: Vec<Spectrum> = rgb.pixels()
        .map(|p| Spectrum::from_srgb(&p.data))
        .collect();

    Ok((pixels, res))
}


#[cfg(not(openexr))]
fn read_image_exr<P: AsRef<Path>>(_path: P) -> Result<(Vec<Spectrum>, Point2i)> {
    panic!("EXR support is not compiled in. Please recompile with the \"openexr\" feature.")
}

#[cfg(openexr)]
fn read_image_exr<P: AsRef<Path>>(path: P) -> Result<(Vec<Spectrum>, Point2i)> {
    info!("Loading EXR texture {}", path.as_ref().display());
    let mut file = File::open(path.as_ref())?;
    let mut exr_file = InputFile::new(&mut file).unwrap();
    let (width, height) = {
        let window = exr_file.header().data_window();
        let width = window.max.x - window.min.x + 1;
        let height = window.max.y - window.min.y + 1;
        (width, height)
    };

    let mut pixel_data: Vec<(f32, f32, f32)> = vec![(0.0, 0.0, 0.0); (width * height) as usize];

    {
        let mut fb = {
            // Create the frame buffer
            let mut fb = FrameBufferMut::new(width as usize, height as usize);
            fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);
            fb
        };

        exr_file.read_pixels(&mut fb).unwrap();
    }

    let pixels = pixel_data
        .iter()
        .map(|&(r, g, b)| Spectrum::rgb(r, g, b))
        .collect();
    Ok((pixels, Point2i::new(width, height)))
}

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\n' || c == '\t'
}

fn read_word<R: BufRead>(f: &mut R) -> String {
    let mut buf = String::new();
    for c in f.bytes() {
        let c = c.unwrap() as char;
        if is_whitespace(c) {
            break;
        }
        buf.push(c);
    }

    buf
}

fn read_image_pfm<P: AsRef<Path>>(path: P) -> Result<(Vec<Spectrum>, Point2i)> {
    info!("Loading PFM file {}", path.as_ref().display());
    let file = File::open(path.as_ref())?;
    let mut reader = BufReader::new(file);
    let mut word;

    word = read_word(&mut reader);
    let n_channels = if &word == "Pf" {
        1
    } else if &word == "PF" {
        3
    } else {
        bail!("Error reading PFM file \"{}\"", path.as_ref().display());
    };

    // Read the rest of the header
    // Read width
    word = read_word(&mut reader);
    let width: usize = word.parse().chain_err(|| "Failed to parse width")?;

    // Read height
    word = read_word(&mut reader);
    let height: usize = word.parse().chain_err(|| "Failed to parse height")?;

    // Read scale
    word = read_word(&mut reader);
    let scale: f32 = word.parse().chain_err(|| "Failed to parse scale")?;
    let file_little_endian = scale < 0.0;
    let host_little_endian = true;

    info!("n_channels={}, width={}, height={}, scale={}",
          n_channels,
          width,
          height,
          scale);

    // Read the rest of the data
    let n_floats = n_channels * width * height;
    let mut data = Vec::new();
    data.resize(n_floats, 0.0f32);
    // Flip in Y, as P*M has the origin at the lower left.
    for y in (0..height).rev() {
        for x in 0..width {
            for idx in 0..n_channels {
                let mut buf = [0u8; 4]; // buffer to read a float
                reader.read_exact(&mut buf)?;
                if host_little_endian ^ file_little_endian {
                    buf.as_mut().swap(0, 3);
                    buf.as_mut().swap(1, 2);
                }
                let mut float = unsafe { ::std::mem::transmute::<[u8; 4], f32>(buf) };
                if scale.abs() != 1.0 {
                    float *= scale.abs();
                }
                data[y * n_channels * width + x * n_channels + idx] = float;
            }
        }
    }

    let rgb: Vec<Spectrum> = if n_channels == 1 {
        data.iter().map(|v| Spectrum::from(*v)).collect()
    } else {
        data.chunks(3)
            .map(|rgb| Spectrum::rgb(rgb[0], rgb[1], rgb[2]))
            .collect()
    };


    Ok((rgb, Point2i::new(width as i32, height as i32)))
}
