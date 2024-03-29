use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use anyhow::*;
use exr;
use image::{self, codecs::hdr::HdrDecoder, GenericImageView};
use log::info;
use rayon::prelude::*;

use crate::bounds::Bounds2i;
use crate::fileutil::has_extension;
use crate::spectrum::{gamma_correct, Spectrum};
use crate::{clamp, Point2i};

pub fn read_image<P: AsRef<Path>>(path: P) -> Result<(Vec<Spectrum>, Point2i), Error> {
    info!("Loading image {}", path.as_ref().display());
    let path = path.as_ref();
    let extension = path
        .extension()
        .ok_or_else(|| format_err!("Texture filename doesn't have an extension"))?;
    if extension == "tga" || extension == "TGA" || extension == "png" || extension == "PNG" {
        read_image_tga_png(path)
    } else if extension == "exr" || extension == "EXR" {
        read_image_exr(path)
    } else if extension == "pfm" {
        read_image_pfm(path)
    } else if extension == "hdr" {
        read_image_hdr(path)
    } else {
        Err(format_err!("Unsupported file format"))
    }
}

pub fn write_image<P: AsRef<Path>>(
    name: P,
    rgb: &[f32],
    output_bounds: &Bounds2i,
    total_resolution: Point2i,
) -> Result<(), Error> {
    let path = name.as_ref();

    if has_extension(path, "png") {
        write_image_png(path, rgb, output_bounds, total_resolution)
    } else if has_extension(path, "exr") {
        write_image_exr(path, rgb, output_bounds, total_resolution)
    } else {
        Err(format_err!("Unsupported file format"))
    }
}

fn write_image_png<P: AsRef<Path>>(
    name: P,
    rgb: &[f32],
    output_bounds: &Bounds2i,
    _total_resolution: Point2i,
) -> Result<(), Error> {
    let path = name.as_ref();
    let resolution = output_bounds.diagonal();
    let rgb8: Vec<_> = rgb
        .iter()
        .map(|v| clamp(255.0 * gamma_correct(*v) + 0.5, 0.0, 255.0) as u8)
        .collect();

    image::save_buffer(
        path,
        &rgb8,
        resolution.x as u32,
        resolution.y as u32,
        image::ColorType::Rgb8,
    )
    .context(format!("Failed to save image file {}", path.display()))?;
    Ok(())
}

fn write_image_exr<P: AsRef<Path>>(
    name: P,
    rgb: &[f32],
    output_bounds: &Bounds2i,
    _total_resolution: Point2i,
) -> Result<(), Error> {
    let path = name.as_ref();
    let resolution = output_bounds.diagonal();
    let (width, height) = (resolution.x as usize, resolution.y as usize);

    exr::prelude::write_rgb_file(path, width, height, |x, y| {
        let offset = y * width + x;
        (rgb[offset * 3], rgb[offset * 3 + 1], rgb[offset * 3 + 2])
    })?;

    Ok(())
}

fn read_image_tga_png<P: AsRef<Path>>(path: P) -> Result<(Vec<Spectrum>, Point2i), Error> {
    info!("Loading texture {}", path.as_ref().display());
    let buf = image::open(path)?;
    let (width, height) = buf.dimensions();

    let rgb = buf.to_rgb8().into_raw();
    let res = Point2i::new(width as i32, height as i32);
    let pixels: Vec<Spectrum> = rgb
        .par_chunks(3)
        .map(|p| {
            let r = f32::from(p[0]) / 255.0;
            let g = f32::from(p[1]) / 255.0;
            let b = f32::from(p[2]) / 255.0;
            Spectrum::rgb(r, g, b)
        })
        .collect();

    Ok((pixels, res))
}

fn read_image_hdr<P: AsRef<Path>>(path: P) -> Result<(Vec<Spectrum>, Point2i), Error> {
    info!("Loading HDR image {}", path.as_ref().display());
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);
    let hdr = HdrDecoder::with_strictness(reader, false)?;

    let meta = hdr.metadata();
    let data = hdr.read_image_hdr()?;

    let data = data
        .into_iter()
        .map(|p| {
            let rgb = p.0;
            Spectrum::rgb(rgb[0], rgb[1], rgb[2])
        })
        .collect();

    Ok((data, Point2i::new(meta.width as i32, meta.height as i32)))
}

fn read_image_exr<P: AsRef<Path>>(path: P) -> Result<(Vec<Spectrum>, Point2i), Error> {
    info!("Loading EXR texture {}", path.as_ref().display());

    let image = exr::prelude::read_first_rgba_layer_from_file(
        path,
        |resolution, _| vec![vec![Spectrum::black(); resolution.width()]; resolution.height()],
        |pixels, pos, (r, g, b, _): (f32, f32, f32, f32)| {
            pixels[pos.y()][pos.x()] = Spectrum::rgb(r, g, b);
        },
    )?;
    let pixels = image
        .layer_data
        .channel_data
        .pixels
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();

    Ok((
        pixels,
        Point2i::new(
            image.attributes.display_window.size.width() as i32,
            image.attributes.display_window.size.height() as i32,
        ),
    ))
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

fn read_image_pfm<P: AsRef<Path>>(path: P) -> Result<(Vec<Spectrum>, Point2i), Error> {
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
    let width: usize = word.parse::<usize>().context("Failed to parse width")?;

    // Read height
    word = read_word(&mut reader);
    let height: usize = word.parse::<usize>().context("Failed to parse height")?;

    // Read scale
    word = read_word(&mut reader);
    let scale: f32 = word.parse::<f32>().context("Failed to parse scale")?;
    let file_little_endian = scale < 0.0;
    let host_little_endian = true;

    info!(
        "n_channels={}, width={}, height={}, scale={}",
        n_channels, width, height, scale
    );

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
