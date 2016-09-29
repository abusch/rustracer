use Dim;
use colour::Colourf;
use filter::Filter;

const FILTER_SIZE: usize = 16;

pub struct Image {
    pub width: usize,
    pub height: usize,
    pixels: Vec<Colourf>,
    samples: Vec<PixelSample>,
    filter_table: Vec<f32>,
    filter: Box<Filter>,
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
        self.c / self.weighted_sum
    }
}

impl Image {
    pub fn new(dim: Dim, filter: Box<Filter>) -> Image {
        let (w, h) = dim;
        let size = w as usize * h as usize;
        let filter_size = FILTER_SIZE * FILTER_SIZE;
        let mut buffer = Vec::with_capacity(size);
        buffer.resize(size, Colourf::black());
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

        Image {
            width: w,
            height: h,
            pixels: buffer,
            samples: samples,
            filter_table: filter_table,
            filter: filter,
        }
    }

    pub fn add_sample(&mut self, x: f32, y: f32, colour: Colourf) {
        // compute sample raster extent (i.e. how many pixels are affected)
        let (xwidth, ywidth) = self.filter.width();
        let (dimagex, dimagey) = (x - 0.5, y - 0.5);
        // (x0, y0) -> (x1, y1) is the zone of the image affected by the sample
        let (x0, y0) = ((dimagex - xwidth).ceil().max(0.0) as usize,
                        (dimagey - ywidth).ceil().max(0.0) as usize);
        let (x1, y1) = ((dimagex + xwidth).floor().min(self.width as f32 - 1.0) as usize,
                        (dimagey + ywidth).floor().min(self.height as f32 - 1.0) as usize);

        // Add this sample's contribution to all the affected pixels
        let (inv_filter_x, inv_filter_y) = self.filter.inv_width();
        for fy in y0..y1 {
            // compute the y-index in the filter table
            let fy_idx = ((fy as f32 - dimagey).abs() * inv_filter_y) as usize;
            for fx in x0..x1 {
                let fx_idx = ((fx as f32 - dimagex).abs() * inv_filter_x) as usize;
                let idx = fy_idx * FILTER_SIZE + fx_idx;
                let pidx = fy * self.width + fx;
                self.samples[pidx].c += colour * self.filter_table[idx];
                self.samples[pidx].weighted_sum += self.filter_table[idx];
            }
        }
    }

    pub fn write(&mut self, x: usize, y: usize, colour: Colourf) {
        self.pixels[y * self.width + x] = colour;
    }

    pub fn buffer(&self) -> &[Colourf] {
        &self.pixels
    }

    pub fn render(&mut self) {
        for s in self.samples.iter().enumerate() {
            self.pixels[s.0] = s.1.render();
        }
    }
}
