use std::f32::consts;
use rand::{thread_rng, Rng};
use ::{Point2f, Vector};
use na::{zero, Vector2};

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
    fn get_samples(&self, x: f32, y: f32, samples: &mut Vec<(f32, f32)>);
    // fn get_2d(&self) -> (f32, f32);
}

pub struct LowDiscrepancy {
    spp: usize,
}

impl LowDiscrepancy {
    pub fn new(spp: usize) -> LowDiscrepancy {
        LowDiscrepancy { spp: spp }
    }
}

impl Sampler for LowDiscrepancy {
    fn get_samples(&self, x: f32, y: f32, samples: &mut Vec<(f32, f32)>) {
        let scramble: (u32, u32) = (thread_rng().gen(), thread_rng().gen());
        for i in 0..self.spp {
            let s = zero_two_sequence(i as u32, scramble);
            samples[i].0 = s.0 + x;
            samples[i].1 = s.1 + y;
        }
    }
}

pub fn zero_two_sequence(n: u32, scramble: (u32, u32)) -> (f32, f32) {
    (van_der_corput(n, scramble.0), sobol(n, scramble.1))
}

fn van_der_corput(n: u32, scramble: u32) -> f32 {
    let mut bits = n;
    bits = (bits << 16) | (bits >> 16);
    bits = ((bits & 0x00ff00ff) << 8) | ((bits & 0xff00ff00) >> 8);
    bits = ((bits & 0x0f0f0f0f) << 4) | ((bits & 0xf0f0f0f0) >> 4);
    bits = ((bits & 0x33333333) << 2) | ((bits & 0xcccccccc) >> 2);
    bits = ((bits & 0x55555555) << 1) | ((bits & 0xaaaaaaaa) >> 1);
    bits ^= scramble;

    (bits >> 8) as f32 / 0x1000000 as f32
}

fn sobol(bits: u32, scramble: u32) -> f32 {
    let mut v: u32 = 1 << 31;
    let mut i = bits;
    let mut r = scramble;

    while i > 0 {
        if i & 1 > 0 {
            r ^= v;
        }
        i >>= 1;
        v ^= v >> 1;
    }

    (r >> 8) as f32 / 0x1000000 as f32
}
