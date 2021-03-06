use std::f32::consts;

use crate::{Point2f, Vector2f, Vector3f};

mod distribution1d;
mod distribution2d;

pub use self::distribution1d::Distribution1D;
pub use self::distribution2d::Distribution2D;

const FRAC_PI_4: f32 = consts::FRAC_PI_2 / 2.0;

// Inline functions
pub fn uniform_sample_sphere(u: Point2f) -> Vector3f {
    let z = 1.0 - 2.0 * u.x;
    let r = (1.0 - z * z).max(0.0).sqrt();
    let phi = 2.0 * consts::PI * u.y;

    Vector3f::new(r * phi.cos(), r * phi.sin(), z)
}

pub fn cosine_sample_hemisphere(u: Point2f) -> Vector3f {
    let d = concentric_sample_disk(u);
    let z = (1.0 - d.x * d.x - d.y * d.y).max(0.0).sqrt();
    Vector3f::new(d.x, d.y, z)
}

pub fn concentric_sample_disk(u: Point2f) -> Point2f {
    // Map uniform random numbers to `[-1, 1]^2`
    let u_offset = 2.0 * u - Vector2f::new(1.0, 1.0);

    // Handle degeneracy at the origin
    if u_offset.x == 0.0 && u_offset.y == 0.0 {
        return Point2f::new(0.0, 0.0);
    }

    // Apply concentric mapping to point
    let (r, theta) = if u_offset.x.abs() > u_offset.y.abs() {
        (u_offset.x, FRAC_PI_4 * (u_offset.y / u_offset.x))
    } else {
        (
            u_offset.y,
            consts::FRAC_PI_2 - FRAC_PI_4 * (u_offset.x / u_offset.y),
        )
    };
    r * Point2f::new(theta.cos(), theta.sin())
}

pub fn uniform_sample_triangle(u: Point2f) -> Point2f {
    let su0 = u[0].sqrt();
    Point2f::new(1.0 - su0, u[1] * su0)
}

pub fn uniform_cone_pdf(cos_theta_max: f32) -> f32 {
    1.0 / (2.0 * consts::PI * (1.0 - cos_theta_max))
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
