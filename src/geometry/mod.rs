use {Vector, Point, Transform};
use ray::Ray;
use na::Norm;

pub use self::sphere::*;
pub use self::plane::*;

mod sphere;
mod plane;

pub struct TextureCoordinate {
    pub u: f32,
    pub v: f32,
}

pub trait Geometry {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry>;
}

pub struct DifferentialGeometry<'a> {
    pub phit: Point,
    pub nhit: Vector,
    pub tex_coord: TextureCoordinate,
    pub geom: &'a Geometry,
}

impl<'a> DifferentialGeometry<'a> {
    pub fn new(p: Point,
               n: Vector,
               tex_coord: TextureCoordinate,
               geom: &Geometry)
               -> DifferentialGeometry {
        DifferentialGeometry {
            phit: p,
            nhit: n,
            tex_coord: tex_coord,
            geom: geom,
        }
    }

    pub fn transform(&mut self, transform: Transform) {
        self.phit = transform * self.phit;
        self.nhit = (transform * self.nhit).normalize();
    }
}
