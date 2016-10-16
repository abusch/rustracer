use {Vector, Point, Transform};
use stats;
use std::f32::INFINITY;
use std::ops::Mul;
use na::Norm;

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
        stats::inc_primary_ray();
        Ray {
            origin: o,
            dir: d,
            t_min: 0.0,
            t_max: INFINITY,
            depth: 0,
        }
    }

    pub fn at(&self, t: f32) -> Point {
        self.origin + t * self.dir
    }

    pub fn spawn(&self, o: Point, d: Vector) -> Ray {
        stats::inc_secondary_ray();
        Ray {
            origin: o,
            dir: d,
            t_min: BIAS,
            t_max: INFINITY,
            depth: self.depth + 1,
        }
    }
}

impl Mul<Ray> for Transform {
    type Output = Ray;

    fn mul(self, rhs: Ray) -> Ray {
        let mut new_ray = rhs;
        new_ray.origin = self * rhs.origin;
        new_ray.dir = (self * rhs.dir).normalize();

        new_ray
    }
}

#[test]
fn test_translation() {
    let r = Ray::new(Point::new(1.0, 0.0, 0.0), Vector::new(0.0, 1.0, 0.0));
    let t = Transform::new(Vector::new(1.0, 1.0, 1.0), Vector::new(0.0, 0.0, 0.0), 1.0);
    let s = t * r;

    assert_eq!(s.origin, Point::new(2.0, 1.0, 1.0));
    assert_eq!(s.dir, r.dir);
}
