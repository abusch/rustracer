#![deny(trivial_casts, unused_qualifications)]
#[macro_use]
extern crate approx;
#[macro_use]
extern crate bitflags;
extern crate image as img;
extern crate itertools as it;
extern crate nalgebra as na;
extern crate rand;
extern crate threadpool as tp;
extern crate ieee754 as fp;
extern crate num;

use na::{Vector3, Point2, Point3, Similarity3, BaseNum};
use std::f32;

mod block_queue;
mod bounds;
mod bsdf;
mod bvh;
pub mod camera;
pub mod spectrum;
pub mod efloat;
mod filter;
pub mod geometry;
mod film;
pub mod instance;
pub mod integrator;
mod interaction;
mod intersection;
pub mod light;
pub mod material;
pub mod primitive;
pub mod ray;
pub mod renderer;
pub mod sampling;
pub mod sampler;
pub mod scene;
pub mod shapes;
mod skydome;
mod stats;
pub mod texture;
mod transform;

pub type Dim = (u32, u32);

pub type Vector = Vector3<f32>;
pub type Point = Point3<f32>;
pub type Point2f = Point2<f32>;
pub type Point2i = Point2<u32>;
pub type Transform = Similarity3<f32>;

pub const MACHINE_EPSILON: f32 = f32::EPSILON * 0.5;
pub fn gamma(n: u32) -> f32 {
    (n as f32 * MACHINE_EPSILON) / (1.0 - n as f32 * MACHINE_EPSILON)
}

/// Linear interpolation between 2 values
pub fn lerp<T: BaseNum>(t: T, a: T, b: T) -> T {
    let one: T = na::one();
    (one - t) * a + t * b
}

#[test]
fn test_gamma() {
    let g5 = gamma(5);
    let p = Point::new(-0.4, 0.9, 0.2);
    let v = g5 * na::abs(&p.to_vector());
    println!("gamma(5) = {}, p={:?}, v={:?}", gamma(5), p, v);
}
