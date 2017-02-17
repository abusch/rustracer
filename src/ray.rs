use {Vector3f, Point3f, Transform};
use stats;
use transform;
use std::f32::INFINITY;
use std::ops::Mul;

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
        assert!(d.norm_squared() != 0.0);
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
        assert!(d.norm_squared() != 0.0);
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
        let (mut o, o_error) = transform::transform_point(transform, &self.o);
        let (d, d_error) = transform::transform_vector(transform, &self.d);
        let t_max = self.t_max;
        let length_squared = d.norm_squared();

        if length_squared > 0.0 {
            let dt = d.abs().dot(&o_error) / length_squared;
            o += d * dt;
        }
        let r = Ray {
            o: o,
            d: d,
            t_max: t_max,
            differential: None,
        };
        (r, o_error, d_error)
    }
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

#[derive(Copy, Clone, Debug)]
pub struct RayDifferential {
    pub rx_origin: Point3f,
    pub ry_origin: Point3f,
    pub rx_direction: Vector3f,
    pub ry_direction: Vector3f,
}

#[test]
fn test_translation() {
    let r = Ray::new(Point3f::new(1.0, 0.0, 0.0), Vector3f::new(0.0, 1.0, 0.0));
    let t = Transform::new(Vector3f::new(1.0, 1.0, 1.0),
                           Vector3f::new(0.0, 0.0, 0.0),
                           1.0);
    let s = t * r;

    assert_eq!(s.o, Point3f::new(2.0, 1.0, 1.0));
    assert_eq!(s.d, r.d);
}
