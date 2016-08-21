use {Vector, Point};
use ray::Ray;

pub trait Geometry {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry>;
}

pub struct DifferentialGeometry<'a> {
    pub phit: Point,
    pub nhit: Vector,
    pub geom: &'a Geometry,
}

impl<'a> DifferentialGeometry<'a> {
    pub fn new(p: Point, n: Vector, geom: &Geometry) -> DifferentialGeometry {
        DifferentialGeometry { phit: p, nhit: n, geom: geom }
    }
}
