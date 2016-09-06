use {Vector, Point};
use std::f32::INFINITY;

const BIAS: f32 = 1e-4;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ray {
    pub origin: Point,
    pub dir: Vector,
    pub t_min: f32,
    pub t_max: f32,
    pub depth: u8,
}

impl Ray {
    pub fn new(o: Point, d: Vector) -> Ray {
        Ray {origin: o, dir: d, t_min: 0.0, t_max: INFINITY, depth: 0}
    }

    pub fn at(&self, t: f32) -> Point {
        self.origin + t * self.dir
    }

    pub fn spawn(&self, o: Point, d: Vector) -> Ray {
        Ray {origin: o, dir: d, t_min: BIAS, t_max: INFINITY, depth: self.depth + 1}
    }
}

