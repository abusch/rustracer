use std::f32::consts;

use na::Vector2;

use {Point2f, Vector};

mod distribution1d;
mod distribution2d;

pub use self::distribution1d::Distribution1D;
pub use self::distribution2d::Distribution2D;

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

#[inline]
pub fn power_heuristic(nf: u32, f_pdf: f32, ng: u32, g_pdf: f32) -> f32 {
    let f = nf as f32 * f_pdf;
    let g = ng as f32 * g_pdf;
    (f * f) / (f * f + g * g)
}

// pub fn zero_two_sequence(n: u32, scramble: (u32, u32)) -> (f32, f32) {
//     (van_der_corput(n, scramble.0), sobol(n, scramble.1))
// }
