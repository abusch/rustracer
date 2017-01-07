use std::f32::consts::PI;

use na;

use ::Vector3f;

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
