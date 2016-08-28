use ray::Ray;
use Point;
use Vector;
use geometry::{Geometry, DifferentialGeometry};
use na::{Norm, Dot};

#[derive(Debug, PartialEq)]
pub struct Plane {
    pub p: Point,
    pub n: Vector,
}

impl Plane {
    pub fn new(p: Point, n: Vector, ) -> Plane {
        Plane {
            p: p,
            n: n.normalize(),
        }
    }
}

impl Geometry for Plane {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        let denom = ray.dir.dot(&self.n);

        if f32::abs(denom) > 1e-6 {
            let p0_l0 = self.p - ray.origin;
            let t = p0_l0.dot(&self.n) / denom;
            if t >= ray.t_min && t <= ray.t_max {
                ray.t_max = t;
                let phit = ray.at(ray.t_max);
                let nhit = if ray.dir.dot(&self.n) > 0.0 { -self.n } else { self.n };
                Some(DifferentialGeometry::new(phit, nhit, self))
            } else {
                None
            }
        } else {
            None
        }
    }
}

