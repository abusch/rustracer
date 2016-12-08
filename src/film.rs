use std::cmp;

use ::{Dim, Point2f, Point2i, Vector2f};
use bounds::{Bounds2i, Bounds2f};
use spectrum::Spectrum;
use filter::Filter;

const FILTER_SIZE: usize = 16;

pub struct Film {
    pub width: u32,
    pub height: u32,
    samples: Vec<PixelSample>,
    filter_table: Vec<f32>,
    filter: Box<Filter + Sync + Send>,
    filter_radius: Vector2f,
}

#[derive(Copy, Clone, Debug)]
pub struct PixelSample {
    pub c: Spectrum,
    pub weighted_sum: f32,
}

impl PixelSample {
    pub fn new() -> PixelSample {
        PixelSample {
            c: Spectrum::black(),
            weighted_sum: 0.0,
        }
    }

    pub fn render(&self) -> Spectrum {
        (self.c / self.weighted_sum)
    }
}

impl Film {
    pub fn new(dim: Dim, filter: Box<Filter + Sync + Send>) -> Film {
        let (w, h) = dim;
        let size = w * h;
        let filter_size = FILTER_SIZE * FILTER_SIZE;
        let mut samples = Vec::with_capacity(size as usize);
        samples.resize(size as usize, PixelSample::new());
        let mut filter_table = Vec::with_capacity(filter_size);
        filter_table.resize(filter_size, 0f32);

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
            samples: samples,
            filter_table: filter_table,
            filter: filter,
            filter_radius: Vector2f::new(xwidth, ywidth),
        }
    }

    pub fn add_sample(&mut self, x: f32, y: f32, colour: Spectrum) {
        if colour.has_nan() {
            warn!("colour has NaNs! Ignoring");
            return;
        }
        let (xwidth, ywidth) = self.filter.width();
        // Convert to discrete pixel space
        let (dimagex, dimagey) = (x - 0.5, y - 0.5);
        // compute sample raster extent (i.e. how many pixels are affected)
        // (x0, y0) -> (x1, y1) is the zone of the image affected by the sample
        let (x0, y0) = ((dimagex - xwidth).ceil().max(0.0) as usize,
                        (dimagey - ywidth).ceil().max(0.0) as usize);
        let (x1, y1) = ((dimagex + xwidth + 1.0).floor().min(self.width as f32) as usize,
                        (dimagey + ywidth + 1.0).floor().min(self.height as f32) as usize);

        // Degenerate case (sample is on or past the image bounds?)
        if x0 >= x1 || y0 >= y1 {
            return;
        }

        let (inv_filter_x, inv_filter_y) = self.filter.inv_width();
        let filter_table_size = FILTER_SIZE as f32;

        // Precompute x and y filter table offset
        let mut ifx = Vec::with_capacity(x1 - x0);
        for x in x0..x1 {
            let fx = ((x as f32 - dimagex) * inv_filter_x * filter_table_size).abs();
            ifx.push(fx.floor().min(filter_table_size - 1.0) as usize);
        }
        let mut ify = Vec::with_capacity(y1 - y0);
        for y in y0..y1 {
            let fy = ((y as f32 - dimagey) * inv_filter_y * filter_table_size).abs();
            ify.push(fy.floor().min(filter_table_size - 1.0) as usize);
        }

        // Add this sample's contribution to all the affected pixels
        for y in y0..y1 {
            for x in x0..x1 {
                let offset = ify[y - y0] * FILTER_SIZE + ifx[x - x0];
                let filter_weight = self.filter_table[offset];
                let pidx = y * self.width as usize + x;
                self.samples[pidx].c += colour * filter_weight;
                self.samples[pidx].weighted_sum += filter_weight;
            }
        }
    }

    pub fn get_film_tile(&self, sample_bounds: &Bounds2i) -> FilmTile {
        let half_pixel = Vector2f::new(0.5, 0.5);
        let float_bounds: Bounds2f = (*sample_bounds).into();
        let p0_f = float_bounds.p_min - half_pixel - self.filter_radius;
        let p0 = Point2i::new(p0_f.x.ceil() as u32, p0_f.y.ceil() as u32);
        let p1_f = float_bounds.p_max - half_pixel - self.filter_radius;
        let p1 = Point2i::new(p1_f.x.ceil() as u32 + 1, p1_f.y.ceil() as u32 + 1);
        let tile_pixel_bounds = Bounds2i::from_points(&p0, &p1);

        FilmTile::new(&tile_pixel_bounds,
                      &self.filter_radius,
                      &self.filter_table[..],
                      FILTER_SIZE * FILTER_SIZE)
    }

    pub fn merge_film_tile(&mut self, tile: FilmTile) {
        for p in &tile.get_pixel_bounds() {
            let pixel = tile.get_pixel(&p);
            let pidx = (p.y * self.width + p.x) as usize;
            self.samples[pidx].c += pixel.contrib_sum;
            self.samples[pidx].weighted_sum += pixel.filter_weight_sum;
        }
    }

    pub fn render(&self) -> Vec<Spectrum> {
        self.samples.iter().map(|s| s.render()).collect()
    }
}

pub struct FilmTile<'a> {
    pixel_bounds: Bounds2i,
    filter_radius: Vector2f,
    inv_filter_radius: Vector2f,
    filter_table: &'a [f32],
    filter_table_size: usize,
    pixels: Vec<FilmTilePixel>,
}

impl<'a> FilmTile<'a> {
    pub fn new(pixel_bounds: &Bounds2i,
               filter_radius: &Vector2f,
               filter: &'a [f32],
               filter_table_size: usize)
               -> FilmTile<'a> {
        FilmTile {
            pixel_bounds: *pixel_bounds,
            filter_radius: *filter_radius,
            inv_filter_radius: Vector2f::new(1.0 / filter_radius.x, 1.0 / filter_radius.y),
            filter_table: filter,
            filter_table_size: filter_table_size,
            pixels: vec![FilmTilePixel::default(); pixel_bounds.get_area() as usize],
        }
    }

    pub fn add_sample(&'a mut self, p_film: &Point2f, colour: Spectrum) {
        if colour.has_nan() {
            println!("WARN: colour has NaNs! Ignoring");
            return;
        }
        // Convert to discrete pixel space
        let p_film_discrete = *p_film - Vector2f::new(0.5, 0.5);
        // compute sample raster extent (i.e. how many pixels are affected)
        // (x0, y0) -> (x1, y1) is the zone of the image affected by the sample
        let p0 = max(&ceil(p_film_discrete - self.filter_radius),
                     &self.pixel_bounds.p_min);
        let p1 = min(&floor(p_film_discrete - self.filter_radius + Vector2f::new(1.0, 1.0)),
                     &self.pixel_bounds.p_max);

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

    pub fn get_pixel(&'a self, p: &Point2i) -> &'a FilmTilePixel {
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

fn max(p1: &Point2i, p2: &Point2i) -> Point2i {
    Point2i::new(cmp::max(p1.x, p2.x), cmp::max(p1.y, p2.y))
}

fn min(p1: &Point2i, p2: &Point2i) -> Point2i {
    Point2i::new(cmp::min(p1.x, p2.x), cmp::min(p1.y, p2.y))
}

fn ceil(p: Point2f) -> Point2i {
    Point2i::new(p.x.ceil() as u32, p.y.ceil() as u32)
}

fn floor(p: Point2f) -> Point2i {
    Point2i::new(p.x.floor() as u32, p.y.floor() as u32)
}
