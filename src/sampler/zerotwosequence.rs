use {Point2i, Point2f};
use camera::CameraSample;
use rng::RNG;
use sampler::Sampler;
use sampler::lowdiscrepancy::{sobol_2d, van_der_corput};

pub struct ZeroTwoSequence {
    spp: usize,
    current_pixel: Point2i,
    current_pixel_sample_index: usize,
    sample_1d_array_sizes: Vec<usize>,
    sample_2d_array_sizes: Vec<usize>,
    sample_array_1d: Vec<Vec<f32>>,
    sample_array_2d: Vec<Vec<Point2f>>,
    array_1d_offset: usize,
    array_2d_offset: usize,
    // Pixel sampler data
    samples_1d: Vec<Vec<f32>>,
    samples_2d: Vec<Vec<Point2f>>,
    current_1d_dimension: usize,
    current_2d_dimension: usize,
    rng: RNG,
}

impl ZeroTwoSequence {
    pub fn new(spp: usize, n_sampled_dimensions: usize) -> ZeroTwoSequence {
        let spp = spp.next_power_of_two();
        let mut samples1d = Vec::with_capacity(n_sampled_dimensions);
        let mut samples2d = Vec::with_capacity(n_sampled_dimensions);
        for _ in 0..n_sampled_dimensions {
            samples1d.push(vec![0.0; spp]);
            samples2d.push(vec![Point2f::new(0.0, 0.0); spp]);
        }

        ZeroTwoSequence {
            spp: spp,
            current_pixel: Point2i::new(0, 0),
            current_pixel_sample_index: 0,
            sample_1d_array_sizes: Vec::new(),
            sample_2d_array_sizes: Vec::new(),
            sample_array_1d: Vec::new(),
            sample_array_2d: Vec::new(),
            array_1d_offset: 0,
            array_2d_offset: 0,
            samples_1d: samples1d,
            samples_2d: samples2d,
            current_1d_dimension: 0,
            current_2d_dimension: 0,
            rng: RNG::new(),
        }
    }
}

impl Sampler for ZeroTwoSequence {
    fn start_pixel(&mut self, p: &Point2i) {
        // Generate 1D and 2D pixel sample components using (0, 2)-sequence
        for i in 0..self.samples_1d.len() {
            van_der_corput(1,
                           self.spp as u32,
                           &mut self.samples_1d[i][..],
                           &mut self.rng);
        }
        for i in 0..self.samples_2d.len() {
            sobol_2d(1,
                     self.spp as u32,
                     &mut self.samples_2d[i][..],
                     &mut self.rng);
        }

        // TODO generate 1d and 2d array samples

        self.current_pixel = *p;
        self.current_pixel_sample_index = 0;
        self.array_1d_offset = 0;
        self.array_2d_offset = 0;
    }

    fn start_next_sample(&mut self) -> bool {
        self.array_1d_offset = 0;
        self.array_2d_offset = 0;
        self.current_1d_dimension = 0;
        self.current_2d_dimension = 0;
        self.current_pixel_sample_index += 1;
        self.current_pixel_sample_index < self.spp
    }

    fn request_1d_array(&mut self, n: usize) {
        self.sample_1d_array_sizes.push(n);
        self.sample_array_1d.push(Vec::with_capacity(n * self.spp));
    }

    fn request_2d_array(&mut self, n: usize) {
        self.sample_2d_array_sizes.push(n);
        self.sample_array_2d.push(Vec::with_capacity(n * self.spp));
    }

    fn get_1d_array(&mut self, n: usize) -> &[f32] {
        assert!(self.array_1d_offset < self.sample_array_1d.len());
        let res = &self.sample_array_1d[self.array_1d_offset][(self.current_pixel_sample_index *
                                                               n)..];
        self.array_1d_offset += 1;
        res
    }

    fn get_2d_array(&mut self, n: usize) -> &[Point2f] {
        assert!(self.array_2d_offset < self.sample_array_2d.len());
        let res = &self.sample_array_2d[self.array_2d_offset][(self.current_pixel_sample_index *
                                                               n)..];
        self.array_2d_offset += 1;
        res
    }

    // fn get_samples(&self, x: f32, y: f32, samples: &mut Vec<(f32, f32)>) {
    //     let scramble: (u32, u32) = (thread_rng().gen(), thread_rng().gen());
    //     for i in 0..self.spp {
    //         let s = zero_two_sequence(i as u32, scramble);
    //         samples[i].0 = s.0 + x;
    //         samples[i].1 = s.1 + y;
    //     }
    // }

    fn get_1d(&mut self) -> f32 {
        if self.current_1d_dimension < self.samples_1d.len() {
            let res = self.samples_1d[self.current_1d_dimension][self.current_pixel_sample_index];
            self.current_1d_dimension += 1;
            res
        } else {
            self.rng.uniform_f32()
        }
    }

    fn get_2d(&mut self) -> Point2f {
        if self.current_2d_dimension < self.samples_2d.len() {
            let res = self.samples_2d[self.current_2d_dimension][self.current_pixel_sample_index];
            self.current_2d_dimension += 1;
            res
        } else {
            Point2f::new(self.rng.uniform_f32(), self.rng.uniform_f32())
        }
    }

    fn get_camera_sample(&mut self, p_raster: &Point2i) -> CameraSample {
        let s = self.get_2d();
        let p_film = Point2f::new(p_raster.x as f32 + s.x, p_raster.y as f32 + s.y);
        let p_lens = self.get_2d();

        CameraSample {
            p_film: p_film,
            p_lens: p_lens,
            time: self.get_1d(),
        }
    }

    fn round_count(&self, count: u32) -> u32 {
        count.next_power_of_two()
    }

    fn reseed(&mut self, seed: u64) {
        self.rng.set_sequence(seed);
    }
}
