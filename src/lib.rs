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

use na::{Vector3, Point3, Similarity3};
use std::f32;

mod block_queue;
mod bsdf;
mod bvh;
pub mod camera;
pub mod colour;
mod efloat;
mod filter;
pub mod geometry;
mod film;
pub mod instance;
pub mod integrator;
mod intersection;
pub mod light;
pub mod material;
mod ray;
pub mod renderer;
mod sampling;
pub mod scene;
pub mod shapes;
mod skydome;
mod stats;


pub fn mix(a: f32, b: f32, mix: f32) -> f32 {
    b * mix + a * (1.0 - mix)
}

pub type Dim = (usize, usize);

pub type Vector = Vector3<f32>;
pub type Point = Point3<f32>;
pub type Transform = Similarity3<f32>;

pub const MACHINE_EPSILON: f32 = f32::EPSILON * 0.5;
pub fn gamma(n: u32) -> f32 {
    (n as f32 * MACHINE_EPSILON) / (1.0 - n as f32 * MACHINE_EPSILON)
}
