use {Vector, Point};
use ray::Ray;
use geometry::*;

#[derive(Debug)]
pub struct Plane;

impl Geometry for Plane {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        if f32::abs(ray.d.z) < 1e-8 {
            return None;
        }

        let t = -ray.o.z / ray.d.z;
        if t < ray.t_min || t > ray.t_max {
            return None;
        }

        let phit = ray.at(t);
        if phit.x >= -1.0 && phit.x <= 1.0 && phit.y >= -1.0 && phit.y <= 1.0 {
            ray.t_max = t;
            Some(DifferentialGeometry::new(phit,
                                           Vector::z(),
                                           TextureCoordinate {
                                               u: phit.x,
                                               v: phit.y,
                                           },
                                           self))
        } else {
            None
        }
    }
}

impl Bounded for Plane {
    fn get_world_bounds(&self) -> BBox {
        BBox::from_points(&Point::new(-1.0, -1.0, 0.0), &Point::new(1.0, 1.0, 0.0))
    }
}
