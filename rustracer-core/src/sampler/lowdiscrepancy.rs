use rng::RNG;

use {Point2i, Point2f, ONE_MINUS_EPSILON};

pub fn van_der_corput(n_samples_per_pixel_sample: u32,
                      n_pixel_samples: u32,
                      samples: &mut [f32],
                      rng: &mut RNG) {
    let scramble: u32 = rng.uniform_u32();
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

pub fn sobol_2d(n_samples_per_pixel_sample: u32,
                n_pixel_samples: u32,
                samples: &mut [Point2f],
                rng: &mut RNG) {
    let scramble = Point2i::new(rng.uniform_u32() as i32, rng.uniform_u32() as i32);

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

pub fn radical_inverse(base: u32, a: u64) -> f32 {
    match base {
        0 => reverse_bits_64(a) as f32 * 5.4210108624275222e-20,
        1 => radical_inverse_specialized(3, a),
        2 => radical_inverse_specialized(5, a),
        3 => radical_inverse_specialized(7, a),
        4 => radical_inverse_specialized(11, a),
        5 => radical_inverse_specialized(13, a),
        _ => unimplemented!(),
    }
}

fn reverse_bits_32(n: u32) -> u32 {
    let mut n = n;
    n = (n << 16) | (n >> 16);
    n = ((n & 0x00ff00ff) << 8) | ((n & 0xff00ff00) >> 8);
    n = ((n & 0x0f0f0f0f) << 4) | ((n & 0xf0f0f0f0) >> 4);
    n = ((n & 0x33333333) << 2) | ((n & 0xcccccccc) >> 2);
    n = ((n & 0x55555555) << 1) | ((n & 0xaaaaaaaa) >> 1);
    n
}

fn reverse_bits_64(n: u64) -> u64 {
    let n0 = reverse_bits_32(n as u32);
    let n1 = reverse_bits_32((n >> 32) as u32);
    ((n0 as u64) << 32) | (n1 as u64)
}

fn radical_inverse_specialized(base: u32, a: u64) -> f32 {
    let mut a = a;
    let inv_base: f32 = 1.0 / base as f32;
    let mut reversed_digits: u64 = 0;
    let mut inv_base_n = 1.0;
    while a != 0 {
        let next = a / base as u64;
        let digit = a - next * base as u64;
        reversed_digits = reversed_digits * base as u64 + digit;
        inv_base_n *= inv_base;
        a = next;
    }
    assert!(reversed_digits as f32 * inv_base_n < 1.00001);
    return f32::min(reversed_digits as f32 * inv_base_n, ONE_MINUS_EPSILON);
}

fn gray_code_sample(c: &[u32], n: u32, scramble: u32, p: &mut [f32]) {
    let mut v = scramble;
    for i in 0..n {
        p[i as usize] = (v as f32 * 2.3283064365386963e-10f32).min(ONE_MINUS_EPSILON);
        v ^= c[(i + 1).trailing_zeros() as usize];
    }
}

fn gray_code_sample_2d(c0: &[u32], c1: &[u32], n: u32, scramble: &Point2i, p: &mut [Point2f]) {
    let mut v = [scramble.x as u32, scramble.y as u32];
    for i in 0..n {
        p[i as usize].x = (v[0] as f32 * 2.3283064365386963e-10f32).min(ONE_MINUS_EPSILON);
        p[i as usize].y = (v[1] as f32 * 2.3283064365386963e-10f32).min(ONE_MINUS_EPSILON);
        v[0] ^= c0[(i + 1).trailing_zeros() as usize];
        v[1] ^= c1[(i + 1).trailing_zeros() as usize];
    }
}

fn shuffle<T>(samp: &mut [T], count: u32, n_dimensions: u32, rng: &mut RNG) {
    for i in 0..count {
        let other: u32 = i + rng.uniform_u32_bounded(count - i);
        for j in 0..n_dimensions {
            samp.swap((n_dimensions * i + j) as usize,
                      (n_dimensions * other + j) as usize);
        }
    }
}

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
