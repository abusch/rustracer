use geometry::{Geometry, DifferentialGeometry, TextureCoordinate};
use ray::Ray;
use Point;
use na::{Dot, Norm, Cross};

pub struct Triangle {
    v0: Point,
    v1: Point,
    v2: Point,
}

impl Triangle {
    pub fn new(v0: Point, v1: Point, v2: Point) -> Triangle {
        Triangle {
            v0: v0,
            v1: v1,
            v2: v2,
        }
    }
}

impl Geometry for Triangle {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        let v0v1 = self.v1 - self.v0;
        let v0v2 = self.v2 - self.v0;
        let pvec = ray.dir.cross(&v0v2);
        let det = v0v1.dot(&pvec);

        if det.abs() < 1e-4 {
            return None;
        }

        let inv_det = 1.0 / det;
        let tvec = ray.origin - self.v0;
        let u = tvec.dot(&pvec) * inv_det;
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let qvec = tvec.cross(&v0v1);
        let v = ray.dir.dot(&qvec) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        ray.t_max = v0v2.dot(&qvec) * inv_det;

        return Some(DifferentialGeometry::new(ray.at(ray.t_max),
                                              v0v1.cross(&v0v2).normalize(),
                                              TextureCoordinate { u: u, v: v },
                                              self));
    }
}
