use {Vector, Point, Transform};
use stats;
use std::f32::INFINITY;
use std::ops::Mul;
use na::{self, ToHomogeneous, Matrix4, Norm, Dot, Absolute};
use ::gamma;

const BIAS: f32 = 1e-4;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ray {
    pub o: Point,
    pub d: Vector,
    // TODO delete?
    pub t_min: f32,
    pub t_max: f32,
}

impl Ray {
    pub fn new(o: Point, d: Vector) -> Ray {
        stats::inc_primary_ray();
        Ray {
            o: o,
            d: d,
            t_min: 0.0,
            t_max: INFINITY,
        }
    }

    pub fn at(&self, t: f32) -> Point {
        self.o + t * self.d
    }

    pub fn spawn(&self, o: Point, d: Vector) -> Ray {
        stats::inc_secondary_ray();
        Ray {
            o: o,
            d: d,
            t_min: BIAS,
            t_max: INFINITY,
        }
    }

    pub fn transform(&self, transform: &Transform) -> (Ray, Vector, Vector) {
        let (mut o, o_error) = transform_point(transform, &self.o);
        let (d, d_error) = transform_vector(transform, &self.d);
        let t_max = self.t_max;
        let length_squared = d.norm_squared();

        if length_squared > 0.0 {
            let dt = na::abs(&d).dot(&o_error) / length_squared;
            o += d * dt;
        }
        let r = Ray {
            o: o,
            d: d,
            t_min: 0.0,
            t_max: t_max,
        };
        (r, o_error, d_error)
    }
}

/// Transform the given point using the given transformation and also return a vector of the
/// absolute error introduced for each coordinate.
fn transform_point(t: &Transform, p: &Point) -> (Point, Vector) {
    let (x, y, z) = (p.x, p.y, p.z);
    let tp = *t * *p;
    let m: Matrix4<f32> = t.to_homogeneous();
    let x_abs_sum = (m[(0, 0)] * x).abs() + (m[(0, 1)] * y).abs() + (m[(0, 2)] * z).abs() +
                    m[(0, 3)].abs();
    let y_abs_sum = (m[(1, 0)] * x).abs() + (m[(1, 1)] * y).abs() + (m[(1, 2)] * z).abs() +
                    m[(1, 3)].abs();
    let z_abs_sum = (m[(2, 0)] * x).abs() + (m[(2, 1)] * y).abs() + (m[(2, 2)] * z).abs() +
                    m[(2, 3)].abs();
    let p_err = gamma(3) * Vector::new(x_abs_sum, y_abs_sum, z_abs_sum);

    (tp, p_err)
}

/// Transform the given point using the given transformation and also return a vector of the
/// absolute error introduced for each coordinate.
fn transform_vector(t: &Transform, v: &Vector) -> (Vector, Vector) {
    let (x, y, z) = (v.x, v.y, v.z);
    let tv = *t * *v;
    let m: Matrix4<f32> = t.to_homogeneous();
    let x_abs_sum = na::abs(&(m[(0, 0)] * x)) + na::abs(&(m[(0, 1)] * y)) +
                    na::abs(&(m[(0, 2)] * z)) + na::abs(&m[(0, 3)]);
    let y_abs_sum = na::abs(&(m[(1, 0)] * x)) + na::abs(&(m[(1, 1)] * y)) +
                    na::abs(&(m[(1, 2)] * z)) + na::abs(&m[(1, 3)]);
    let z_abs_sum = na::abs(&(m[(2, 0)] * x)) + na::abs(&(m[(2, 1)] * y)) +
                    na::abs(&(m[(2, 2)] * z)) + na::abs(&m[(2, 3)]);
    let v_err = gamma(3) * Vector::new(x_abs_sum, y_abs_sum, z_abs_sum);

    (tv, v_err)
}

impl Mul<Ray> for Transform {
    type Output = Ray;

    fn mul(self, rhs: Ray) -> Ray {
        let mut new_ray = rhs;
        new_ray.o = self * rhs.o;
        new_ray.d = self * rhs.d;

        new_ray
    }
}

#[test]
fn test_translation() {
    let r = Ray::new(Point::new(1.0, 0.0, 0.0), Vector::new(0.0, 1.0, 0.0));
    let t = Transform::new(Vector::new(1.0, 1.0, 1.0), Vector::new(0.0, 0.0, 0.0), 1.0);
    let s = t * r;

    assert_eq!(s.o, Point::new(2.0, 1.0, 1.0));
    assert_eq!(s.d, r.d);
}
