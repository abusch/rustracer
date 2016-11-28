use std::f32::consts;

use rand::{thread_rng, Rng};
use na::Vector2;

use {Point2i, Point2f, Vector};

const FRAC_PI_4: f32 = consts::FRAC_PI_2 / 2.0;

// Inline functions
pub fn uniform_sample_sphere(u: &Point2f) -> Vector {
    let z = 1.0 - 2.0 * u.x;
    let r = (1.0 - z * z).max(0.0).sqrt();
    let phi = 2.0 * consts::PI * u.y;

    Vector::new(r * phi.cos(), r * phi.sin(), z)
}

pub fn cosine_sample_hemisphere(u: &Point2f) -> Vector {
    let d = concentric_sample_disk(u);
    let z = (1.0 - d.x * d.x - d.y * d.y).max(0.0).sqrt();
    Vector::new(d.x, d.y, z)
}

pub fn concentric_sample_disk(u: &Point2f) -> Point2f {
    let u_offset = 2.0 * *u - Vector2::<f32>::new(1.0, 1.0);
    if u_offset.x == 0.0 && u_offset.y == 0.0 {
        return Point2f::new(0.0, 0.0);
    }

    let (r, theta) = if u_offset.x.abs() > u_offset.y.abs() {
        (u_offset.x, FRAC_PI_4 * (u_offset.y / u_offset.x))
    } else {
        (u_offset.y, consts::FRAC_PI_2 - FRAC_PI_4 * (u_offset.x / u_offset.y))
    };
    r * Point2f::new(theta.cos(), theta.sin())
}

pub trait Sampler {
    fn start_pixel(&mut self, p: &Point2i);
    // fn get_samples(&self, x: f32, y: f32, samples: &mut Vec<(f32, f32)>);
    fn get_1d(&mut self) -> f32;
    fn get_2d(&mut self) -> Point2f;
    fn get_camera_sample(&mut self) -> Point2f;
    fn request_1d_array(&mut self, n: usize);
    fn request_2d_array(&mut self, n: usize);
    fn round_count(&self, count: u32) -> u32;
    fn get_1d_array(&mut self, n: usize) -> &[f32];
    fn get_2d_array(&mut self, n: usize) -> &[Point2f];
    fn start_next_sample(&mut self) -> bool;
}

pub struct LowDiscrepancy {
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
}

impl LowDiscrepancy {
    pub fn new(spp: usize, n_sampled_dimensions: usize) -> LowDiscrepancy {
        let spp = spp.next_power_of_two();
        let mut samples1d = Vec::with_capacity(n_sampled_dimensions);
        let mut samples2d = Vec::with_capacity(n_sampled_dimensions);
        for _ in 0..n_sampled_dimensions {
            samples1d.push(vec![0.0; spp]);
            samples2d.push(vec![Point2f::new(0.0, 0.0); spp]);
        }

        LowDiscrepancy {
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
        }
    }
}

impl Sampler for LowDiscrepancy {
    fn start_pixel(&mut self, p: &Point2i) {
        let mut rng = thread_rng();
        // Generate 1D and 2D pixel sample components using (0, 2)-sequence
        for i in 0..self.samples_1d.len() {
            van_der_corput(1, self.spp as u32, &mut self.samples_1d[i][..], &mut rng);
        }
        for i in 0..self.samples_2d.len() {
            sobol_2d(1, self.spp as u32, &mut self.samples_2d[i][..], &mut rng);
        }

        // TODO generate 1d and 2d array samples
        // println!("Generated samples for pixel: {:?}", p);
        // println!("1d samples: {:?}", self.samples_1d);
        // println!("2d samples: {:?}", self.samples_2d);

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
            thread_rng().next_f32()
        }
    }

    fn get_2d(&mut self) -> Point2f {
        if self.current_2d_dimension < self.samples_2d.len() {
            let res = self.samples_2d[self.current_2d_dimension][self.current_pixel_sample_index];
            self.current_2d_dimension += 1;
            res
        } else {
            Point2f::new(thread_rng().next_f32(), thread_rng().next_f32())
        }
    }

    fn get_camera_sample(&mut self) -> Point2f {
        let s = self.get_2d();
        Point2f::new(self.current_pixel.x as f32 + s.x,
                     self.current_pixel.y as f32 + s.y)
    }

    fn round_count(&self, count: u32) -> u32 {
        count.next_power_of_two()
    }
}

// pub fn zero_two_sequence(n: u32, scramble: (u32, u32)) -> (f32, f32) {
//     (van_der_corput(n, scramble.0), sobol(n, scramble.1))
// }

fn van_der_corput<T: Rng>(n_samples_per_pixel_sample: u32,
                          n_pixel_samples: u32,
                          samples: &mut [f32],
                          rng: &mut T) {
    let scramble: u32 = rng.gen();
    let total_samples = n_samples_per_pixel_sample * n_pixel_samples;
    gray_code_sample(&CVAN_DER_CORPUT, total_samples, scramble, &mut samples[..]);
    // Randomly shuffle 1D points
    for i in 0..n_pixel_samples {
        shuffle(&mut samples[(i as usize * n_samples_per_pixel_sample as usize)..],
                n_samples_per_pixel_sample,
                1,
                rng);
    }
    shuffle(&mut samples[..],
            n_pixel_samples,
            n_samples_per_pixel_sample,
            rng);
}

fn gray_code_sample(C: &[u32], n: u32, scramble: u32, p: &mut [f32]) {
    let mut v = scramble;
    for i in 0..n {
        p[i as usize] = (v as f32 * 2.3283064365386963e-10f32).min(ONE_MINUS_EPSILON);
        v ^= C[(i + 1).trailing_zeros() as usize];
    }
}

fn gray_code_sample_2d(C0: &[u32], C1: &[u32], n: u32, scramble: &Point2i, p: &mut [Point2f]) {
    let mut v = [scramble.x, scramble.y];
    for i in 0..n {
        p[i as usize].x = (v[0] as f32 * 2.3283064365386963e-10f32).min(ONE_MINUS_EPSILON);
        p[i as usize].y = (v[1] as f32 * 2.3283064365386963e-10f32).min(ONE_MINUS_EPSILON);
        v[0] ^= C0[(i + 1).trailing_zeros() as usize];
        v[1] ^= C1[(i + 1).trailing_zeros() as usize];
    }
}

fn shuffle<R: Rng, T>(samp: &mut [T], count: u32, n_dimensions: u32, rng: &mut R) {
    for i in 0..count {
        let other: u32 = i + rng.gen_range(0, count - i);
        for j in 0..n_dimensions {
            samp.swap((n_dimensions * i + j) as usize,
                      (n_dimensions * other + j) as usize);
        }
    }
}

fn sobol_2d<T: Rng>(n_samples_per_pixel_sample: u32,
                    n_pixel_samples: u32,
                    samples: &mut [Point2f],
                    rng: &mut T) {
    let scramble = Point2i::new(rng.gen(), rng.gen());

    gray_code_sample_2d(&CSOBOL[0],
                        &CSOBOL[1],
                        n_samples_per_pixel_sample * n_pixel_samples,
                        &scramble,
                        &mut samples[..]);
    // Randomly shuffle 2D points
    for i in 0..n_pixel_samples {
        shuffle(&mut samples[(i as usize * n_samples_per_pixel_sample as usize)..],
                n_samples_per_pixel_sample,
                1,
                rng);
    }
    shuffle(&mut samples[..],
            n_pixel_samples,
            n_samples_per_pixel_sample,
            rng);

}


// fn van_der_corput(n: u32, scramble: u32) -> f32 {
//     let mut bits = n;
//     bits = (bits << 16) | (bits >> 16);
//     bits = ((bits & 0x00ff00ff) << 8) | ((bits & 0xff00ff00) >> 8);
//     bits = ((bits & 0x0f0f0f0f) << 4) | ((bits & 0xf0f0f0f0) >> 4);
//     bits = ((bits & 0x33333333) << 2) | ((bits & 0xcccccccc) >> 2);
//     bits = ((bits & 0x55555555) << 1) | ((bits & 0xaaaaaaaa) >> 1);
//     bits ^= scramble;

//     (bits >> 8) as f32 / 0x1000000 as f32
// }

// fn sobol(bits: u32, scramble: u32) -> f32 {
//     let mut v: u32 = 1 << 31;
//     let mut i = bits;
//     let mut r = scramble;

//     while i > 0 {
//         if i & 1 > 0 {
//             r ^= v;
//         }
//         i >>= 1;
//         v ^= v >> 1;
//     }

//     (r >> 8) as f32 / 0x1000000 as f32
// }

/// Smallest representable float strictly less than 1
const ONE_MINUS_EPSILON: f32 = 0.99999994f32;
const CVAN_DER_CORPUT: [u32; 32] = [0b_10000000000000000000000000000000,
                                    0b_1000000000000000000000000000000,
                                    0b_100000000000000000000000000000,
                                    0b_10000000000000000000000000000,
                                    0b_1000000000000000000000000000,
                                    0b_100000000000000000000000000,
                                    0b_10000000000000000000000000,
                                    0b_1000000000000000000000000,
                                    0b_100000000000000000000000,
                                    0b_10000000000000000000000,
                                    0b_1000000000000000000000,
                                    0b_100000000000000000000,
                                    0b_10000000000000000000,
                                    0b_1000000000000000000,
                                    0b_100000000000000000,
                                    0b_10000000000000000,
                                    0b_1000000000000000,
                                    0b_100000000000000,
                                    0b_10000000000000,
                                    0b_1000000000000,
                                    0b_100000000000,
                                    0b_10000000000,
                                    0b_1000000000,
                                    0b_100000000,
                                    0b_10000000,
                                    0b_1000000,
                                    0b_100000,
                                    0b_10000,
                                    0b_1000,
                                    0b_100,
                                    0b_10,
                                    0b_1];
/// Generator matrices for Sobol 2D
const CSOBOL: [[u32; 32]; 2] =
    [[0x80000000, 0x40000000, 0x20000000, 0x10000000, 0x8000000, 0x4000000, 0x2000000, 0x1000000,
      0x800000, 0x400000, 0x200000, 0x100000, 0x80000, 0x40000, 0x20000, 0x10000, 0x8000, 0x4000,
      0x2000, 0x1000, 0x800, 0x400, 0x200, 0x100, 0x80, 0x40, 0x20, 0x10, 0x8, 0x4, 0x2, 0x1],
     [0x80000000, 0xc0000000, 0xa0000000, 0xf0000000, 0x88000000, 0xcc000000, 0xaa000000,
      0xff000000, 0x80800000, 0xc0c00000, 0xa0a00000, 0xf0f00000, 0x88880000, 0xcccc0000,
      0xaaaa0000, 0xffff0000, 0x80008000, 0xc000c000, 0xa000a000, 0xf000f000, 0x88008800,
      0xcc00cc00, 0xaa00aa00, 0xff00ff00, 0x80808080, 0xc0c0c0c0, 0xa0a0a0a0, 0xf0f0f0f0,
      0x88888888, 0xcccccccc, 0xaaaaaaaa, 0xffffffff]];
