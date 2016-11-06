use std::sync::Arc;

use Vector;
use colour::Colourf;
use geometry::DifferentialGeometry;
use material::Material;

pub struct Intersection<'a> {
    pub dg: DifferentialGeometry<'a>,
    pub wo: Vector,
    pub material: Arc<Material>,
}

impl<'a> Intersection<'a> {
    pub fn new(dg: DifferentialGeometry, wo: Vector, material: Arc<Material>) -> Intersection {
        Intersection {
            dg: dg,
            wo: wo,
            material: material,
        }
    }

    pub fn le(&self, wo: Vector) -> Colourf {
        Colourf::black()
    }
}
