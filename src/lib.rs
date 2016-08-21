extern crate nalgebra as na;

use na::{Vector3, Point3};

pub mod ray;
pub mod sphere;
pub mod intersection;
pub mod scene;
pub mod colour;
pub mod camera;
pub mod image;
pub mod geometry;
pub mod instance;
pub mod material;
pub mod light;


pub fn mix(a: f32, b: f32, mix: f32) -> f32 {
    b*mix + a*(1.0 - mix)
}

pub type Dim = (u32, u32);

pub type Vector = Vector3<f32>;
pub type Point = Point3<f32>;
