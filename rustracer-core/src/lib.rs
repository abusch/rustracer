#![recursion_limit = "128"]
// Lints
#![deny(unused_qualifications, unused_must_use)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![allow(non_snake_case)]
// Clippy config
#![allow(
    clippy::float_cmp,
    clippy::many_single_char_names,
    clippy::if_same_then_else,
    clippy::excessive_precision,
    clippy::too_many_arguments,
    clippy::suspicious_operation_groupings
)]

use std::f32;
use std::ops::{Add, Mul, Sub};

use num::{Num, One, Signed};

// stats needs to be declared first so its macros can be used by the following modules
#[macro_use]
mod stats;
mod api;
mod blockedarray;
pub mod bounds;
mod bsdf;
pub mod bvh;
pub mod camera;
mod cie;
pub mod efloat;
mod fileutil;
pub mod film;
pub mod filter;
mod floatfile;
mod geometry;
pub mod imageio;
pub mod integrator;
mod interaction;
mod interpolation;
pub mod light;
pub mod lightdistrib;
pub mod material;
pub mod mipmap;
mod noise;
mod paramset;
pub mod pbrt;
pub mod primitive;
pub mod ray;
pub mod renderer;
pub mod rng;
pub mod sampler;
pub mod sampling;
pub mod scene;
pub mod shapes;
pub mod spectrum;
pub mod texture;
pub mod transform;

pub fn init_stats() {
    // This one needs to be called first
    stats::init_stats();
    api::init_stats();
    bvh::init_stats();
    film::init_stats();
    integrator::init_stats();
    lightdistrib::init_stats();
    mipmap::init_stats();
    renderer::init_stats();
    scene::init_stats();
    shapes::init_stats();
}

use geometry::{Normal3, Point2, Point3, Vector2, Vector3};
use spectrum::Spectrum;

pub type Vector2f = Vector2<f32>;
pub type Vector3f = Vector3<f32>;
pub type Point2f = Point2<f32>;
pub type Point2i = Point2<i32>;
pub type Point3f = Point3<f32>;
pub type Point3i = Point3<i32>;
pub type Normal3f = Normal3<f32>;

pub use transform::Transform;

pub const INV_2_PI: f32 = 0.15915494309189533577;
pub const MACHINE_EPSILON: f32 = f32::EPSILON * 0.5;
pub fn gamma(n: u32) -> f32 {
    (n as f32 * MACHINE_EPSILON) / (1.0 - n as f32 * MACHINE_EPSILON)
}

/// Smallest representable float strictly less than 1
pub const ONE_MINUS_EPSILON: f32 = 0.99999994f32;

#[derive(Debug, Copy, Clone, Default)]
pub struct PbrtOptions {
    pub num_threads: u8,
    pub quick_render: bool,
}

/// Linear interpolation between 2 values.
///
/// This version should be generic enough to linearly interpolate between 2 Spectrums using an f32
/// parameter.
pub fn lerp<S, T>(t: S, a: T, b: T) -> T
where
    S: One,
    S: Sub<S, Output = S>,
    S: Copy,
    T: Add<T, Output = T>,
    T: Mul<S, Output = T>,
{
    let one: S = num::one();
    a * (one - t) + b * t
}

/// Return the dimension index (0, 1 or 2) that contains the largest component.
pub fn max_dimension<T>(v: &Vector3<T>) -> usize
where
    T: Num + PartialOrd,
{
    if v.x > v.y {
        if v.x > v.z {
            0
        } else {
            2
        }
    } else if v.y > v.z {
        1
    } else {
        2
    }
}

pub fn max_component(v: &Vector3f) -> f32 {
    f32::max(v.x, f32::max(v.y, v.z))
}

/// Permute the components of this vector based on the given indices for x, y and z.
pub fn permute_v<T>(v: &Vector3<T>, x: usize, y: usize, z: usize) -> Vector3<T>
where
    T: Num + Copy,
{
    Vector3::new(v[x], v[y], v[z])
}

/// Permute the components of this point based on the given indices for x, y and z.
pub fn permute_p<T>(v: &Point3<T>, x: usize, y: usize, z: usize) -> Point3<T>
where
    T: Num + Signed + Copy,
{
    Point3::new(v[x], v[y], v[z])
}

/// Create an orthogonal coordinate system from a single vector.
pub fn coordinate_system(v1: &Vector3f) -> (Vector3f, Vector3f) {
    let v2 = if v1.x.abs() > v1.y.abs() {
        Vector3::new(-v1.z, 0.0, v1.x) / (v1.x * v1.x + v1.z * v1.z).sqrt()
    } else {
        Vector3::new(0.0, v1.z, -v1.y) / (v1.y * v1.y + v1.z * v1.z).sqrt()
    };

    let v3 = v1.cross(&v2);

    (v2, v3)
}

// TODO does this exist in std?
pub fn find_interval<P>(size: usize, pred: P) -> usize
where
    P: Fn(usize) -> bool,
{
    let mut first = 0;
    let mut len = size;
    while len > 0 {
        let half = len >> 1;
        let middle = first + half;
        // Bisect range based on value of _pred_ at _middle_
        if pred(middle as usize) {
            first = middle + 1;
            len -= half + 1;
        } else {
            len = half;
        }
    }
    clamp(first as isize - 1, 0, size as isize - 2) as usize
}

/// Version of min() that works on `PartialOrd`, so it works for both u32 and f32.
pub fn min<T: PartialOrd + Copy>(a: T, b: T) -> T {
    if a.lt(&b) {
        a
    } else {
        b
    }
}

/// Version of max() that works on `PartialOrd`, so it works for both u32 and f32.
pub fn max<T: PartialOrd + Copy>(a: T, b: T) -> T {
    if a.gt(&b) {
        a
    } else {
        b
    }
}

#[inline]
pub fn is_power_of_2(v: i32) -> bool {
    (v != 0) && (v & (v - 1)) == 0
}

#[inline]
pub fn round_up_pow_2(v: i32) -> i32 {
    let mut v = v;
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v + 1
}

#[inline]
pub fn next_float_up(v: f32) -> f32 {
    let mut v = v;
    if v.is_infinite() && v > 0.0 {
        return v;
    }

    if v == -0.0 {
        v = 0.0;
    }
    let mut ui = v.to_bits();
    if v >= 0.0 {
        ui += 1;
    } else {
        ui -= 1;
    }
    f32::from_bits(ui)
}

#[inline]
pub fn next_float_down(v: f32) -> f32 {
    let mut v = v;
    if v.is_infinite() && v < 0.0 {
        return v;
    }

    if v == 0.0 {
        v = -0.0;
    }
    let mut ui = v.to_bits();
    if v > 0.0 {
        ui -= 1;
    } else {
        ui += 1;
    }
    f32::from_bits(ui)
}

pub fn clamp<T>(val: T, low: T, high: T) -> T
where
    T: PartialOrd + Copy,
{
    if val < low {
        low
    } else if val > high {
        high
    } else {
        val
    }
}

pub trait Clampable {
    fn clamp(self, min: f32, max: f32) -> Self;
}

impl Clampable for f32 {
    fn clamp(self, min: f32, max: f32) -> f32 {
        clamp(self, min, max)
    }
}

impl Clampable for Spectrum {
    fn clamp(self, min: f32, max: f32) -> Spectrum {
        Spectrum::rgb(
            Clampable::clamp(self.r, min, max),
            Clampable::clamp(self.g, min, max),
            Clampable::clamp(self.b, min, max),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_interval() {
        let a = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

        // check clamping for out of range
        assert_eq!(0, find_interval(a.len(), |index| a[index] as isize <= -1));
        assert_eq!(a.len() - 2, find_interval(a.len(), |index| a[index] <= 100));

        for i in 0..a.len() - 1 {
            assert_eq!(i, find_interval(a.len(), |index| a[index] <= i));
            assert_eq!(
                i,
                find_interval(a.len(), |index| a[index] as f32 <= i as f32 + 0.5)
            );
            if i > 0 {
                assert_eq!(
                    i - 1,
                    find_interval(a.len(), |index| a[index] as f32 <= i as f32 - 0.5)
                );
            }
        }
    }

    #[test]
    fn test_gamma() {
        let g5 = gamma(5);
        let p = Point3f::new(-0.4, 0.9, 0.2);
        let v = g5 * p.abs();
        println!("gamma(5) = {}, p={:?}, v={:?}", gamma(5), p, v);
    }

    #[test]
    fn test_is_power_of_2() {
        assert!(is_power_of_2(4));
        assert!(is_power_of_2(8));
        assert!(is_power_of_2(1024));
        assert!(!is_power_of_2(3));
        assert!(!is_power_of_2(7));
    }

    #[test]
    fn test_round_up_pow_2() {
        assert_eq!(round_up_pow_2(1023), 1024);
        assert_eq!(round_up_pow_2(1024), 1024);
    }
}
