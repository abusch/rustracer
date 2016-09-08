use Point;
use ray::Ray;
use geometry::*;
use na::{Norm, Dot, origin};
use std::f32::consts::FRAC_1_PI;

#[derive(Debug, PartialEq)]
pub struct Sphere {
    radius: f32,
    radius_2: f32,
}

impl Sphere {
    pub fn new(r: f32, ) -> Sphere {
        Sphere {
            radius: r,
            radius_2: r*r,
        }
    }
}

impl Geometry for Sphere {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        let l = origin::<Point>() - ray.origin;
        let tca = l.dot(&ray.dir);
        if tca < 0.0 {
            return None;
        }
        let d2 = l.dot(&l) - tca * tca;
        if d2 > self.radius_2 {
            return None;
        }
        let thc = f32::sqrt(self.radius_2 - d2);

        let (t0, t1) = match (tca - thc, tca + thc) {
            (x, y) if x < y => (x, y),
            (x, y)          => (y, x)
        };

        if t1 < ray.t_min || t0 > ray.t_max {
            return None;
        }

        let t_hit;
        if t0 >= ray.t_min {
            t_hit = t0;
        } else if t1 <= ray.t_max {
            t_hit = t1;
        } else {
            return None;
        }

        ray.t_max = t_hit;

        let phit = ray.at(ray.t_max);
        let nhit = phit.to_vector().normalize();
        let phi = f32::atan2(phit.z, phit.x);
        let theta = f32::acos(phit.y / self.radius);
        let u = if phi < 0.0 {
            phi * FRAC_1_PI + 1.0
        } else {
            phi * FRAC_1_PI
        };
        let v = theta * FRAC_1_PI;
        Some(DifferentialGeometry::new(phit, nhit, TextureCoordinate { u: u, v: v}, self))
    }
}
