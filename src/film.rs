use Dim;
use colour::Colourf;
use filter::Filter;

const FILTER_SIZE: usize = 16;

pub struct Film {
    pub width: usize,
    pub height: usize,
    samples: Vec<PixelSample>,
    filter_table: Vec<f32>,
    filter: Box<Filter + Sync + Send>,
}

#[derive(Copy, Clone, Debug)]
pub struct PixelSample {
    pub c: Colourf,
    pub weighted_sum: f32,
}

impl PixelSample {
    pub fn new() -> PixelSample {
        PixelSample {
            c: Colourf::black(),
            weighted_sum: 0.0,
        }
    }

    pub fn render(&self) -> Colourf {
        (self.c / self.weighted_sum)
    }
}

impl Film {
    pub fn new(dim: Dim, filter: Box<Filter + Sync + Send>) -> Film {
        let (w, h) = dim;
        let size = w as usize * h as usize;
        let filter_size = FILTER_SIZE * FILTER_SIZE;
        let mut samples = Vec::with_capacity(size);
        samples.resize(size, PixelSample::new());
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
        }
    }

    pub fn add_sample(&mut self, x: f32, y: f32, colour: Colourf) {
        if colour.has_nan() {
            println!("WARN: colour has NaNs! Ignoring");
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
                let pidx = y * self.width + x;
                self.samples[pidx].c += colour * filter_weight;
                self.samples[pidx].weighted_sum += filter_weight;
            }
        }
    }

    pub fn render(&self) -> Vec<Colourf> {
        self.samples.iter().map(|s| s.render()).collect()
    }
}
