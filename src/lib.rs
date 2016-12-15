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

use std::f32;
use std::ops::{Add, Mul, Sub};

use na::{Vector2, Vector3, Point2, Point3, Similarity3, BaseNum};
use num::One;

mod blockedarray;
mod block_queue;
mod bounds;
mod bsdf;
// mod bvh;
pub mod camera;
mod cie;
pub mod spectrum;
pub mod efloat;
mod filter;
// pub mod geometry;
mod film;
// pub mod instance;
pub mod integrator;
mod interaction;
// mod intersection;
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

pub type Vector = Vector3<f32>;
pub type Vector2f = Vector2<f32>;
pub type Vector3f = Vector3<f32>;
pub type Point = Point3<f32>;
pub type Point2f = Point2<f32>;
pub type Point2i = Point2<u32>;
pub type Point3f = Point3<f32>;
pub type Transform = Similarity3<f32>;

pub const MACHINE_EPSILON: f32 = f32::EPSILON * 0.5;
pub fn gamma(n: u32) -> f32 {
    (n as f32 * MACHINE_EPSILON) / (1.0 - n as f32 * MACHINE_EPSILON)
}

/// Linear interpolation between 2 values
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

pub fn max_dimension<T>(v: Vector3<T>) -> usize
    where T: PartialOrd
{
    if v.x > v.y {
        if v.x > v.z { 0 } else { 2 }
    } else {
        if v.y > v.z { 1 } else { 2 }
    }
}

pub fn permute_v<T>(v: &Vector3<T>, x: usize, y: usize, z: usize) -> Vector3<T>
    where T: Copy
{
    Vector3::new(v[x], v[y], v[z])
}

pub fn permute_p<T>(v: &Point3<T>, x: usize, y: usize, z: usize) -> Point3<T>
    where T: Copy
{
    Point3::new(v[x], v[y], v[z])
}

#[test]
fn test_gamma() {
    let g5 = gamma(5);
    let p = Point::new(-0.4, 0.9, 0.2);
    let v = g5 * na::abs(&p.to_vector());
    println!("gamma(5) = {}, p={:?}, v={:?}", gamma(5), p, v);
}
