use std::f32::INFINITY;
use std::ops::Mul;
use std::fmt;

use num::zero;

use {Point3f, Transform, Vector3f};
use stats;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub o: Point3f,
    pub d: Vector3f,
    pub t_max: f32,
    pub differential: Option<RayDifferential>,
}

impl Ray {
    pub fn new(o: Point3f, d: Vector3f) -> Ray {
        stats::inc_primary_ray();
        assert!(!o.x.is_nan() && !o.y.is_nan() && !o.z.is_nan());
        assert!(!d.x.is_nan() && !d.y.is_nan() && !d.z.is_nan());
        assert_ne!(d.length_squared(), 0.0);
        Ray {
            o: o,
            d: d,
            t_max: INFINITY,
            differential: None,
        }
    }

    pub fn segment(o: Point3f, d: Vector3f, tmax: f32) -> Ray {
        stats::inc_primary_ray();
        assert!(!o.x.is_nan() && !o.y.is_nan() && !o.z.is_nan());
        assert!(!d.x.is_nan() && !d.y.is_nan() && !d.z.is_nan());
        assert_ne!(d.length_squared(), 0.0);
        Ray {
            o: o,
            d: d,
            t_max: tmax,
            differential: None,
        }
    }

    pub fn at(&self, t: f32) -> Point3f {
        self.o + t * self.d
    }

    pub fn transform(&self, transform: &Transform) -> (Ray, Vector3f, Vector3f) {
        let (mut o, o_error) = transform.transform_point(&self.o);
        let (d, d_error) = transform.transform_vector(&self.d);
        let t_max = self.t_max;
        let length_squared = d.length_squared();

        if length_squared > 0.0 {
            let dt = d.abs().dot(&o_error) / length_squared;
            o += d * dt;
        }

        let diff = self.differential
            .map(|d| {
                     RayDifferential {
                         rx_origin: transform * &d.rx_origin,
                         ry_origin: transform * &d.ry_origin,
                         rx_direction: transform * &d.rx_direction,
                         ry_direction: transform * &d.ry_direction,
                     }
                 });

        let r = Ray {
            o: o,
            d: d,
            t_max: t_max,
            differential: diff,
        };
        (r, o_error, d_error)
    }

    pub fn scale_differentials(&mut self, s: f32) {
        if let Some(d) = self.differential.iter_mut().next() {
            d.rx_origin = self.o + (d.rx_origin - self.o) * s;
            d.ry_origin = self.o + (d.ry_origin - self.o) * s;
            d.rx_direction = self.d + (d.rx_direction - self.d) * s;
            d.ry_direction = self.d + (d.ry_direction - self.d) * s;
        }
    }
}

impl Mul<Ray> for Transform {
    type Output = Ray;

    fn mul(self, rhs: Ray) -> Ray {
        let mut new_ray = rhs;
        new_ray.o = &self * &rhs.o;
        new_ray.d = &self * &rhs.d;

        new_ray
    }
}

impl fmt::Display for Ray {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "[o={}, d={}, t_max={}]", self.o, self.d, self.t_max)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RayDifferential {
    pub rx_origin: Point3f,
    pub ry_origin: Point3f,
    pub rx_direction: Vector3f,
    pub ry_direction: Vector3f,
}

impl Default for RayDifferential {
    fn default() -> Self {
        RayDifferential {
            rx_origin: zero(),
            ry_origin: zero(),
            rx_direction: zero(),
            ry_direction: zero(),
        }
    }
}

#[test]
fn test_translation() {
    let r = Ray::new(Point3f::new(1.0, 0.0, 0.0), Vector3f::new(0.0, 1.0, 0.0));
    let t = Transform::translate(&Vector3f::new(1.0, 1.0, 1.0));
    let s = t * r;

    assert_eq!(s.o, Point3f::new(2.0, 1.0, 1.0));
    assert_eq!(s.d, r.d);
}
