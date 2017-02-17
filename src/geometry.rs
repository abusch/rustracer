use std::f32::consts::PI;

use na;

use Vector3f;

// Common geometric functions
#[inline]
pub fn cos_theta(w: &Vector3f) -> f32 {
    w.z
}

#[inline]
pub fn cos2_theta(w: &Vector3f) -> f32 {
    w.z * w.z
}

#[inline]
pub fn abs_cos_theta(w: &Vector3f) -> f32 {
    w.z.abs()
}

#[inline]
pub fn sin2_theta(w: &Vector3f) -> f32 {
    (1.0 - cos2_theta(w)).max(0.0)
}

#[inline]
pub fn sin_theta(w: &Vector3f) -> f32 {
    sin2_theta(w).sqrt()
}

#[inline]
pub fn tan_theta(w: &Vector3f) -> f32 {
    sin_theta(w) / cos_theta(w)
}

#[inline]
pub fn tan2_theta(w: &Vector3f) -> f32 {
    sin2_theta(w) / cos2_theta(w)
}

#[inline]
pub fn cos_phi(w: &Vector3f) -> f32 {
    let sin_theta = sin_theta(w);
    if sin_theta == 0.0 {
        0.0
    } else {
        na::clamp(w.x / sin_theta, -1.0, 1.0)
    }
}

#[inline]
pub fn sin_phi(w: &Vector3f) -> f32 {
    let sin_theta = sin_theta(w);
    if sin_theta == 0.0 {
        0.0
    } else {
        na::clamp(w.y / sin_theta, -1.0, 1.0)
    }
}

#[inline]
pub fn cos2_phi(w: &Vector3f) -> f32 {
    cos_phi(w) * cos_phi(w)
}

#[inline]
pub fn sin2_phi(w: &Vector3f) -> f32 {
    sin_phi(w) * sin_phi(w)
}

#[inline]
pub fn cos_d_phi(wa: &Vector3f, wb: &Vector3f) -> f32 {
    na::clamp((wa.x * wb.x + wa.y * wa.y) /
              ((wa.x * wa.x + wa.y * wa.y) * (wb.x * wb.x + wb.y * wb.y)).sqrt(),
              -1.0,
              1.0)
}

#[inline]
pub fn same_hemisphere(w: &Vector3f, wp: &Vector3f) -> bool {
    w.z * wp.z > 0.0
}

#[inline]
pub fn spherical_theta(v: &Vector3f) -> f32 {
    na::clamp(v.z, -1.0, 1.0).acos()
}

#[inline]
pub fn spherical_phi(v: &Vector3f) -> f32 {
    let p = v.y.atan2(v.x);
    if p < 0.0 { p + 2.0 * PI } else { p }
}

#[inline]
pub fn spherical_direction(sin_theta: f32, cos_theta: f32, phi: f32) -> Vector3f {
    Vector3f::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta)
}

#[inline]
pub fn face_forward(v1: &Vector3f, v2: &Vector3f) -> Vector3f {
    if v1.dot(v2) < 0.0 { -(*v1) } else { *v1 }
}


/// Polynomial approximation of the inverse Gauss error function
#[inline]
pub fn erf_inv(x: f32) -> f32 {
    let x = na::clamp(x, -0.99999, 0.99999);
    let mut w = -((1.0 - x) * (1.0 + x)).ln();
    let mut p;
    if w < 5.0 {
        w = w - 2.5;
        p = 2.81022636e-08;
        p = 3.43273939e-07 + p * w;
        p = -3.5233877e-06 + p * w;
        p = -4.39150654e-06 + p * w;
        p = 0.00021858087 + p * w;
        p = -0.00125372503 + p * w;
        p = -0.00417768164 + p * w;
        p = 0.246640727 + p * w;
        p = 1.50140941 + p * w;
    } else {
        w = w.sqrt() - 3.0;
        p = -0.000200214257;
        p = 0.000100950558 + p * w;
        p = 0.00134934322 + p * w;
        p = -0.00367342844 + p * w;
        p = 0.00573950773 + p * w;
        p = -0.0076224613 + p * w;
        p = 0.00943887047 + p * w;
        p = 1.00167406 + p * w;
        p = 2.83297682 + p * w;
    }

    p * x
}

/// Polynomial approximation of the Gauss error function.
///
/// See [https://en.wikipedia.org/wiki/Error_function]
pub fn erf(x: f32) -> f32 {
    // constants
    const a1: f32 = 0.254829592;
    const a2: f32 = -0.284496736;
    const a3: f32 = 1.421413741;
    const a4: f32 = -1.453152027;
    const a5: f32 = 1.061405429;
    const p: f32 = 0.3275911;

    // Save the sign of x
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    // A&S formula 7.1.26
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}
