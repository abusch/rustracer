use std::f32::INFINITY;
use vector::Vector;
use point::Point;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ray {
    pub origin: Point,
    pub dir: Vector,
    pub t_min: f32,
    pub t_max: f32,
}

impl Ray {
    pub fn new(o: Point, d: Vector) -> Ray {
        Ray {origin: o, dir: d, t_min: 0.0,  t_max: INFINITY}
    }

    pub fn at(&self, t: f32) -> Point {
        (self.origin + t * self.dir).as_point()
    }
}

