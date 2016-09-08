use Vector;
use ray::Ray;
use geometry::*;

#[derive(Debug, PartialEq)]
pub struct Plane;

impl Geometry for Plane {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        if f32::abs(ray.dir.z) > 1e-6 {
            let t = -ray.origin.z / ray.dir.z;
            if t >= ray.t_min && t <= ray.t_max {
                ray.t_max = t;
                let phit = ray.at(ray.t_max);
                Some(DifferentialGeometry::new(phit, Vector::new(0.0, 0.0, 1.0), TextureCoordinate { u: 0.0, v: 0.0}, self))
            } else {
                None
            }
        } else {
            None
        }
    }
}

