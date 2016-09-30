use rand::{thread_rng, Rng};

pub trait Sampler {
    fn get_samples(&self, x: f32, y: f32, samples: &mut Vec<(f32, f32)>);
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
