use Vector;
use colour::Colourf;
use geometry::DifferentialGeometry;

pub struct Intersection<'a> {
    pub dg: DifferentialGeometry<'a>,
}

impl<'a> Intersection<'a> {
    pub fn new(dg: DifferentialGeometry) -> Intersection {
        Intersection { dg: dg }
    }

    pub fn le(&self, wo: Vector) -> Colourf {
        Colourf::black()
    }
}
