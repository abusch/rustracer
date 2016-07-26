use geometry::DifferentialGeometry;
use instance::Instance;

pub struct Intersection<'a, 'b> {
    pub dg: DifferentialGeometry<'a>,
    pub hit: &'b Instance,
}

impl<'a, 'b> Intersection<'a, 'b> {
    pub fn new(dg: DifferentialGeometry<'a>, o: &'b Instance) -> Intersection<'a, 'b> {
        Intersection {dg: dg, hit: o}
    }
}
