use std::sync::Mutex;

use na::Point2;

use {Dim, Point2f, Point2i, Vector2f, min, max};
use bounds::{Bounds2i, Bounds2f};
use spectrum::Spectrum;
use filter::Filter;

const FILTER_SIZE: usize = 16;
const FILTER_TABLE_SIZE: usize = FILTER_SIZE * FILTER_SIZE;

pub struct Film {
    pub width: u32,
    pub height: u32,
    samples: Mutex<Vec<PixelSample>>,
    filter_table: [f32; FILTER_TABLE_SIZE],
    filter_radius: Vector2f,
    cropped_pixel_bounds: Bounds2i,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PixelSample {
    pub c: Spectrum,
    pub weighted_sum: f32,
}

impl PixelSample {
    pub fn render(&self) -> Spectrum {
        if self.weighted_sum == 0.0 {
            Spectrum::black()
        } else {
            self.c / self.weighted_sum
        }
    }
}

impl Film {
    pub fn new(dim: Dim, filter: Box<Filter + Sync + Send>) -> Film {
        let (w, h) = dim;
        let size = w * h;
        let mut samples = Vec::with_capacity(size as usize);
        samples.resize(size as usize, PixelSample::default());
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
            width: w,
            height: h,
            samples: Mutex::new(samples),
            filter_table: filter_table,
            filter_radius: Vector2f::new(xwidth, ywidth),
            cropped_pixel_bounds: Bounds2i::from_points(&Point2i::new(0, 0),
                                                        &Point2i::new(dim.0, dim.1)),
        }
    }

    pub fn get_film_tile(&self, sample_bounds: &Bounds2i) -> FilmTile {
        let half_pixel = Vector2f::new(0.5, 0.5);
        let float_bounds: Bounds2f = (*sample_bounds).into();
        let float_cropped_pixel_bounds: Bounds2f = self.cropped_pixel_bounds.into();

        // This is a bit clunky but we need to do all the computations as floats as the numbers can
        // temporarily be negative which would cause u32 to wrap around.
        let p0 = ceil(float_bounds.p_min - half_pixel - self.filter_radius);
        let p1 = floor(float_bounds.p_max - half_pixel - self.filter_radius +
                       Vector2f::new(1.0, 1.0));
        let sample_extent_bounds = Bounds2f::from_points(&p0, &p1);

        let tile_pixel_bounds: Bounds2i =
            Bounds2i::from(Bounds2f::intersect(&sample_extent_bounds, &float_cropped_pixel_bounds));

        FilmTile::new(&tile_pixel_bounds, &self.filter_radius, &self.filter_table)
    }

    pub fn merge_film_tile(&self, tile: FilmTile) {
        let mut samples = self.samples.lock().unwrap();
        for p in &tile.get_pixel_bounds() {
            let pixel = tile.get_pixel(&p);
            let pidx = (p.y * self.width + p.x) as usize;
            samples[pidx].c += pixel.contrib_sum;
            samples[pidx].weighted_sum += pixel.filter_weight_sum;
        }
    }

    pub fn render(&self) -> Vec<Spectrum> {
        let samples = self.samples.lock().unwrap();
        samples.iter().map(|s| s.render()).collect()
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
            pixels: vec![FilmTilePixel::default(); pixel_bounds.get_area() as usize],
        }
    }

    pub fn add_sample(&mut self, p_film: &Point2f, colour: Spectrum) {
        if colour.has_nan() {
            println!("WARN: colour has NaNs! Ignoring");
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
                let ref mut pixel = self.pixels[idx];
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
