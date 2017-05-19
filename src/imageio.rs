use std::path::Path;

use img;
use openexr::{InputFile, FrameBuffer};

use Point2i;
use errors::*;
use spectrum::Spectrum;

pub fn read_image<P: AsRef<Path>>(path: P) -> Result<(Vec<Spectrum>, Point2i)> {
    info!("Loading image {}", path.as_ref().display());
    let path = path.as_ref();
    let extension = path.extension()
        .ok_or("Texture filename doesn't have an extension")?;
    if extension == "tga" || extension == "TGA" {
        read_image_tga(path)
    } else if extension == "exr" || extension == "EXR" {
        read_image_exr(path)
    } else {
        bail!("Unsupported file format");
    }
}

fn read_image_tga<P: AsRef<Path>>(path: P) -> Result<(Vec<Spectrum>, Point2i)> {
    info!("Loading TGA texture {}", path.as_ref().display());
    let buf = img::open(path)?;

    let rgb = buf.to_rgb();
    let res = Point2i::new(rgb.width() as i32, rgb.height() as i32);
    let pixels: Vec<Spectrum> = rgb.pixels()
        .map(|p| Spectrum::from_srgb(&p.data))
        .collect();

    Ok((pixels, res))
}

fn read_image_exr<P: AsRef<Path>>(path: P) -> Result<(Vec<Spectrum>, Point2i)> {
    info!("Loading EXR texture {}", path.as_ref().display());
    let re = InputFile::from_file(path.as_ref())?;
    let window = re.data_window();
    let width = window.max.x - window.min.x + 1;
    let height = window.max.y - window.min.y + 1;

    let mut pixel_data: Vec<(f32, f32, f32)> = vec![(0.0, 0.0, 0.0); (width*height) as usize];

    {
        let mut fb = {
            // Create the frame buffer
            let mut fb = FrameBuffer::new(width as usize, height as usize);
            fb.insert_pixels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);
            fb
        };

        re.read_pixels(&mut fb).unwrap();
    }

    let pixels = pixel_data
        .iter()
        .map(|&(r, g, b)| Spectrum::rgb(r, g, b))
        .collect();
    Ok((pixels, Point2i::new(width, height)))


}
