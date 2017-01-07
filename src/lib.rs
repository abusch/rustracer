#![deny(trivial_casts, unused_qualifications)]
#[macro_use]
extern crate approx;
#[macro_use]
extern crate bitflags;
extern crate crossbeam;
extern crate ieee754 as fp;
extern crate image as img;
extern crate itertools as it;
extern crate nalgebra as na;
extern crate num;
extern crate rand;
#[macro_use(o, slog_info, slog_debug, slog_warn, slog_error, slog_trace, slog_log)]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate uuid;

use std::f32;
use std::ops::{Add, Mul, Sub};

use na::{Vector2, Vector3, Point2, Point3, Similarity3, Cross};
use num::One;

mod blockedarray;
mod block_queue;
pub mod bounds;
mod bsdf;
pub mod bvh;
pub mod camera;
mod cie;
pub mod spectrum;
pub mod efloat;
mod filter;
mod film;
mod geometry;
pub mod integrator;
mod interaction;
pub mod light;
pub mod material;
pub mod mipmap;
pub mod primitive;
pub mod ray;
pub mod renderer;
pub mod sampling;
pub mod sampler;
pub mod scene;
pub mod shapes;
// mod skydome;
mod stats;
pub mod texture;
pub mod transform;

pub type Dim = (u32, u32);

pub type Vector2f = Vector2<f32>;
pub type Vector3f = Vector3<f32>;
pub type Point2f = Point2<f32>;
pub type Point2i = Point2<u32>;
pub type Point3f = Point3<f32>;
pub type Transform = Similarity3<f32>;

pub const MACHINE_EPSILON: f32 = f32::EPSILON * 0.5;
pub fn gamma(n: u32) -> f32 {
    (n as f32 * MACHINE_EPSILON) / (1.0 - n as f32 * MACHINE_EPSILON)
}

/// Smallest representable float strictly less than 1
pub const ONE_MINUS_EPSILON: f32 = 0.99999994f32;

/// Linear interpolation between 2 values.
pub fn lerp<S, T>(t: S, a: T, b: T) -> T
    where S: One,
          S: Sub<S, Output = S>,
          S: Copy,
          T: Add<T, Output = T>,
          T: Mul<S, Output = T>
{
    let one: S = num::one();
    a * (one - t) + b * t
}

/// Return the dimension index (0, 1 or 2) that contains the largest component.
pub fn max_dimension<T>(v: Vector3<T>) -> usize
    where T: PartialOrd
{
    if v.x > v.y {
        if v.x > v.z { 0 } else { 2 }
    } else if v.y > v.z {
        1
    } else {
        2
    }
}

/// Permute the components of this vector based on the given indices for x, y and z.
pub fn permute_v<T>(v: &Vector3<T>, x: usize, y: usize, z: usize) -> Vector3<T>
    where T: Copy
{
    Vector3::new(v[x], v[y], v[z])
}

/// Permute the components of this point based on the given indices for x, y and z.
pub fn permute_p<T>(v: &Point3<T>, x: usize, y: usize, z: usize) -> Point3<T>
    where T: Copy
{
    Point3::new(v[x], v[y], v[z])
}

/// Created an orthogonal coordinate system from a single vector.
pub fn coordinate_system(v1: &Vector3f) -> (Vector3f, Vector3f) {
    let v2 = if v1.x.abs() > v1.y.abs() {
        Vector3::new(-v1.z, 0.0, v1.x) / (v1.x * v1.x + v1.z * v1.z).sqrt()
    } else {
        Vector3::new(0.0, v1.z, v1.y) / (v1.y * v1.y + v1.z * v1.z).sqrt()
    };

    let v3 = v1.cross(&v2);

    (v2, v3)
}

// TODO does this exist in std?
pub fn find_interval<P>(size: usize, pred: P) -> usize
    where P: Fn(usize) -> bool
{
    let mut first = 0;
    let mut len = size;
    while len > 0 {
        let half = len >> 1;
        let middle = first + half;
        // Bisect range based on value of _pred_ at _middle_
        if pred(middle) {
            first = middle + 1;
            len -= half + 1;
        } else {
            len = half;
        }
    }
    na::clamp(first - 1, 0, size - 2)
}

#[test]
fn test_gamma() {
    let g5 = gamma(5);
    let p = Point3f::new(-0.4, 0.9, 0.2);
    let v = g5 * na::abs(&p.to_vector());
    println!("gamma(5) = {}, p={:?}, v={:?}", gamma(5), p, v);
}
