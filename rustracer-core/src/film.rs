use std::sync::atomic::{Ordering, AtomicU32};
use std::f32;

use parking_lot::Mutex;

use {clamp, Point2f, Point2i, Vector2f};
use bounds::{Bounds2f, Bounds2i};
use errors::*;
use filter::Filter;
use imageio;
use paramset::ParamSet;
use spectrum::Spectrum;

const FILTER_SIZE: usize = 16;
const FILTER_TABLE_SIZE: usize = FILTER_SIZE * FILTER_SIZE;

#[derive(Default)]
struct AtomicFloat {
    bits: AtomicU32,
}

impl AtomicFloat {
    pub fn new(v: f32) -> AtomicFloat {
        AtomicFloat { bits: AtomicU32::new(v.to_bits()) }
    }

    pub fn as_float(&self) -> f32 {
        f32::from_bits(self.bits.load(Ordering::Relaxed))
    }
}

#[derive(Default)]
struct Pixel {
    xyz: [f32; 3],
    filter_weight_sum: f32,
    splat_xyz: [AtomicFloat; 3],
    _pad: f32,
}

pub struct Film {
    pub full_resolution: Point2i,
    pub _diagonal: f32,
    pub filename: String,
    pub cropped_pixel_bounds: Bounds2i,
    pixels: Mutex<Vec<Pixel>>,
    filter_table: [f32; FILTER_TABLE_SIZE],
    filter_radius: Vector2f,
    scale: f32,
}

impl Film {
    pub fn new(resolution: Point2i,
               cropwindow: Bounds2f,
               filter: Box<Filter + Sync + Send>,
               diagonal: f32,
               filename: &str,
               scale: f32)
               -> Film {
        let cropped_pixel_bounds =
            Bounds2i::from_points(&Point2i::new((resolution.x as f32 * cropwindow.p_min.x).ceil() as
                                                i32,
                                                (resolution.y as f32 * cropwindow.p_min.y).ceil() as
                                                i32),
                                  &Point2i::new((resolution.x as f32 * cropwindow.p_max.x).ceil() as
                                                i32,
                                                (resolution.y as f32 * cropwindow.p_max.y).ceil() as
                                                i32));

        info!("Created film with full resolution {}. Crop window of {} -> cropped_pixel_bounds {}",
              resolution,
              cropwindow,
              cropped_pixel_bounds);
        let mut pixels = Vec::with_capacity(cropped_pixel_bounds.area() as usize);
        pixels.resize_default(cropped_pixel_bounds.area() as usize);
        let mut filter_table = [0f32; FILTER_TABLE_SIZE];

        let (xwidth, ywidth) = filter.width();
        // Fill in filter table
        for y in 0..FILTER_SIZE {
            let fy = (y as f32 + 0.5) * (ywidth / FILTER_SIZE as f32);
            for x in 0..FILTER_SIZE {
                let fx = (x as f32 + 0.5) * (xwidth / FILTER_SIZE as f32);
                filter_table[y * FILTER_SIZE + x] = filter.evaluate(fx, fy);
            }
        }

        Film {
            full_resolution: resolution,
            pixels: Mutex::new(pixels),
            filter_table: filter_table,
            filter_radius: Vector2f::new(xwidth, ywidth),
            cropped_pixel_bounds: cropped_pixel_bounds,
            scale: scale,
            _diagonal: diagonal * 0.001,
            filename: filename.to_owned(),
        }
    }

    pub fn create(ps: &mut ParamSet, filter: Box<Filter + Send + Sync>) -> Box<Film> {
        let mut filename = ps.find_one_string("filename", "".into());
        if filename == "" {
            filename = "image.png".into();
        } else {
            filename = String::from("rt-") + &filename;
        }
        let xres = ps.find_one_int("xresolution", 1280);
        let yres = ps.find_one_int("yresolution", 720);
        let mut crop = Bounds2f::from_points(&Point2f::new(0.0, 0.0), &Point2f::new(1.0, 1.0));
        if let Some(cr) = ps.find_float("cropwindow") {
            if cr.len() == 4 {
                crop.p_min.x = clamp(f32::min(cr[0], cr[1]), 0.0, 1.0);
                crop.p_max.x = clamp(f32::max(cr[0], cr[1]), 0.0, 1.0);
                crop.p_min.y = clamp(f32::min(cr[2], cr[3]), 0.0, 1.0);
                crop.p_max.y = clamp(f32::max(cr[2], cr[3]), 0.0, 1.0);
            } else {
                warn!("\"cropwindow\" expected 4 values");
            }
        }
        let scale = ps.find_one_float("scale", 1.0);
        let diagonal = ps.find_one_float("diagonal", 35.0);
        // TODO max_sample_luminance
        Box::new(Film::new(Point2i::new(xres, yres),
                           crop,
                           filter,
                           diagonal,
                           &filename,
                           scale))
    }

    pub fn get_film_tile(&self, sample_bounds: &Bounds2i) -> FilmTile {
        let half_pixel = Vector2f::new(0.5, 0.5);
        let float_bounds: Bounds2f = (*sample_bounds).into();
        let float_cropped_pixel_bounds: Bounds2f = self.cropped_pixel_bounds.into();

        // This is a bit clunky but we need to do all the computations as floats as the numbers can
        // temporarily be negative which would cause u32 to wrap around.
        let p0 = ceil(float_bounds.p_min - half_pixel - self.filter_radius);
        let p1 = floor(float_bounds.p_max - half_pixel + self.filter_radius +
                       Vector2f::new(1.0, 1.0));
        let sample_extent_bounds = Bounds2f::from_points(&p0, &p1);

        let tile_pixel_bounds: Bounds2i =
            Bounds2i::from(Bounds2f::intersect(&sample_extent_bounds, &float_cropped_pixel_bounds));

        FilmTile::new(&tile_pixel_bounds, &self.filter_radius, &self.filter_table)
    }

    pub fn merge_film_tile(&self, tile: FilmTile) {
        let mut pixels = self.pixels.lock();
        for pixel in &tile.get_pixel_bounds() {
            let tile_pixel = tile.get_pixel(&pixel);
            let pidx = {
                let width = self.cropped_pixel_bounds.p_max.x - self.cropped_pixel_bounds.p_min.x;
                ((pixel.y - self.cropped_pixel_bounds.p_min.y) * width +
                 (pixel.x - self.cropped_pixel_bounds.p_min.x)) as usize
            };
            let xyz = tile_pixel.contrib_sum.to_xyz();
            for i in 0..3 {
                pixels[pidx].xyz[i] += xyz[i];
            }
            pixels[pidx].filter_weight_sum += tile_pixel.filter_weight_sum;
        }
    }

    pub fn write_image(&self) -> Result<()> {
        info!("Converting image to RGB and computing final weighted pixel values");
        let splat_scale = 1.0; // TODO
        let pixels = self.pixels.lock();
        let mut rgb = Vec::with_capacity(3 * self.cropped_pixel_bounds.area() as usize);
        for p in &self.cropped_pixel_bounds {
            // Convert pixel XYZ color to RGB
            let pixel_idx = self.get_pixel_idx(&p);
            let pixel = &pixels[pixel_idx];
            let mut rgb_pixel = Spectrum::from_xyz(&pixel.xyz);

            // Normalize pixel with weight sum
            let filter_weight_sum = pixel.filter_weight_sum;
            if filter_weight_sum != 0.0 {
                let inv_wt = 1.0 / filter_weight_sum;
                rgb_pixel[0] = f32::max(0.0, rgb_pixel[0] * inv_wt);
                rgb_pixel[1] = f32::max(0.0, rgb_pixel[1] * inv_wt);
                rgb_pixel[2] = f32::max(0.0, rgb_pixel[2] * inv_wt);
            }

            let splat_xyz = [pixel.splat_xyz[0].as_float(),
                             pixel.splat_xyz[1].as_float(),
                             pixel.splat_xyz[2].as_float()];
            let mut splat_rgb = Spectrum::from_xyz(&splat_xyz);
            rgb_pixel[0] += splat_scale * splat_rgb[0];
            rgb_pixel[1] += splat_scale * splat_rgb[1];
            rgb_pixel[2] += splat_scale * splat_rgb[2];

            // Scale pixel value by scale
            rgb_pixel[0] *= self.scale;
            rgb_pixel[1] *= self.scale;
            rgb_pixel[2] *= self.scale;

            rgb.push(rgb_pixel[0]);
            rgb.push(rgb_pixel[1]);
            rgb.push(rgb_pixel[2]);
        }

        // Write RGB image
        info!("Writing image {} with bounds {}",
              self.filename,
              self.cropped_pixel_bounds);
        imageio::write_image(&self.filename,
                             &rgb[..],
                             &self.cropped_pixel_bounds,
                             &self.full_resolution)
    }

    pub fn get_sample_bounds(&self) -> Bounds2i {
        let half = Vector2f::new(0.5, 0.5);
        let float_bounds =
            Bounds2f::from_points(&floor(Point2f::from(self.cropped_pixel_bounds.p_min) + half -
                                         self.filter_radius),
                                  &ceil(Point2f::from(self.cropped_pixel_bounds.p_max) - half +
                                        self.filter_radius));

        float_bounds.into()
    }

    fn get_pixel_idx(&self, p: &Point2i) -> usize {
        assert!(self.cropped_pixel_bounds.inside_exclusive(p));
        let width = self.cropped_pixel_bounds.p_max.x - self.cropped_pixel_bounds.p_min.x;
        let offset = (p.x - self.cropped_pixel_bounds.p_min.x) +
                     (p.y - self.cropped_pixel_bounds.p_min.y) * width;
        offset as usize
    }
}

pub struct FilmTile {
    pixel_bounds: Bounds2i,
    filter_radius: Vector2f,
    inv_filter_radius: Vector2f,
    filter_table: Box<[f32]>,
    pub pixels: Vec<FilmTilePixel>,
}

impl FilmTile {
    pub fn new(pixel_bounds: &Bounds2i, filter_radius: &Vector2f, filter: &[f32]) -> FilmTile {
        let mut filter_table = Vec::new();
        filter_table.extend_from_slice(filter);
        FilmTile {
            pixel_bounds: *pixel_bounds,
            filter_radius: *filter_radius,
            inv_filter_radius: Vector2f::new(1.0 / filter_radius.x, 1.0 / filter_radius.y),
            // Duplicating the filter table in every table is wasteful, but keeping a reference to
            // the data from Film leads to all kind of lifetime issues...
            filter_table: filter_table.into_boxed_slice(),
            pixels: vec![FilmTilePixel::default(); pixel_bounds.area() as usize],
        }
    }

    pub fn add_sample(&mut self, p_film: &Point2f, colour: Spectrum) {
        if colour.has_nan() {
            warn!("colour has NaNs! Ignoring");
            return;
        }
        let float_pixel_bounds: Bounds2f = self.pixel_bounds.into();
        // Convert to discrete pixel space
        let p_film_discrete = *p_film - Vector2f::new(0.5, 0.5);
        // compute sample raster extent (i.e. how many pixels are affected)
        // (x0, y0) -> (x1, y1) is the zone of the image affected by the sample
        let p0_f = ceil(p_film_discrete - self.filter_radius);

        let p1_f = floor(p_film_discrete + self.filter_radius + Vector2f::new(1.0, 1.0));

        let bounds: Bounds2i = Bounds2i::from(Bounds2f::intersect(&Bounds2f::from_points(&p0_f,
                                                                                         &p1_f),
                                                                  &float_pixel_bounds));
        let (p0, p1) = (bounds.p_min, bounds.p_max);

        assert!(p1.x >= p0.x && p1.y >= p0.y,
                format!("p_film={}, p0={}, p1={}, pixel_bounds={:?}",
                        p_film,
                        p0,
                        p1,
                        self.pixel_bounds));

        let filter_table_size = FILTER_SIZE as f32;

        // Precompute x and y filter table offset
        let mut ifx = Vec::with_capacity(p1.x as usize - p0.x as usize);
        for x in p0.x..p1.x {
            let fx = ((x as f32 - p_film_discrete.x) * self.inv_filter_radius.x *
                      filter_table_size)
                    .abs();
            ifx.push(fx.floor().min(filter_table_size - 1.0) as usize);
        }
        let mut ify = Vec::with_capacity(p1.y as usize - p0.y as usize);
        for y in p0.y..p1.y {
            let fy = ((y as f32 - p_film_discrete.y) * self.inv_filter_radius.y *
                      filter_table_size)
                    .abs();
            ify.push(fy.floor().min(filter_table_size - 1.0) as usize);
        }

        // Add this sample's contribution to all the affected pixels
        for y in p0.y..p1.y {
            for x in p0.x..p1.x {
                let offset = ify[(y - p0.y) as usize] * FILTER_SIZE + ifx[(x - p0.x) as usize];
                let filter_weight = &self.filter_table[offset];
                let idx = self.get_pixel_index(&Point2i::new(x, y));
                let pixel = &mut self.pixels[idx];
                pixel.contrib_sum += colour * *filter_weight;
                pixel.filter_weight_sum += *filter_weight;
            }
        }
    }

    pub fn get_pixel<'a>(&'a self, p: &Point2i) -> &'a FilmTilePixel {
        &self.pixels[self.get_pixel_index(p)]
    }

    pub fn get_pixel_bounds(&self) -> Bounds2i {
        self.pixel_bounds
    }

    fn get_pixel_index(&self, p: &Point2i) -> usize {
        let width = self.pixel_bounds.p_max.x - self.pixel_bounds.p_min.x;
        let pidx = (p.y - self.pixel_bounds.p_min.y) * width + (p.x - self.pixel_bounds.p_min.x);
        pidx as usize
    }
}

#[derive(Clone, Default)]
pub struct FilmTilePixel {
    contrib_sum: Spectrum,
    filter_weight_sum: f32,
}

fn ceil(p: Point2f) -> Point2f {
    Point2f::new(p.x.ceil(), p.y.ceil())
}

fn floor(p: Point2f) -> Point2f {
    Point2f::new(p.x.floor(), p.y.floor())
}
