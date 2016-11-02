#[macro_use]
extern crate approx;
extern crate image as img;
extern crate nalgebra as na;
extern crate rand;
extern crate threadpool as tp;
extern crate bitflags;
extern crate itertools as it;

use na::{Vector3, Point3, Similarity3};

mod block_queue;
pub mod bvh;
pub mod camera;
pub mod colour;
pub mod filter;
pub mod geometry;
pub mod film;
pub mod instance;
pub mod integrator;
pub mod intersection;
pub mod light;
pub mod material;
pub mod ray;
pub mod renderer;
pub mod sampling;
pub mod scene;
pub mod skydome;
pub mod stats;


pub fn mix(a: f32, b: f32, mix: f32) -> f32 {
    b * mix + a * (1.0 - mix)
}

pub type Dim = (usize, usize);

pub type Vector = Vector3<f32>;
pub type Point = Point3<f32>;
pub type Transform = Similarity3<f32>;
