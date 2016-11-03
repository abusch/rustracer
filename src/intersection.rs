use Vector;
use bsdf::BSDF;
use colour::Colourf;
use geometry::DifferentialGeometry;

pub struct Intersection<'a> {
    pub dg: DifferentialGeometry<'a>,
    pub wo: Vector,
    pub bsdf: BSDF,
}

impl<'a> Intersection<'a> {
    pub fn new(dg: DifferentialGeometry, wo: Vector) -> Intersection {
        let n = dg.nhit;
        Intersection {
            dg: dg,
            wo: wo,
            bsdf: BSDF::new(n),
        }
    }

    pub fn le(&self, wo: Vector) -> Colourf {
        Colourf::black()
    }
}
