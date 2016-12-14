use {Vector, Point, Transform};
use ray::Ray;
use transform;
use na::Norm;

pub use self::bbox::*;
pub use self::sphere::*;
// pub use self::triangle::*;
pub use self::mesh::*;

mod mesh;
mod bbox;
mod sphere;

pub struct TextureCoordinate {
    pub u: f32,
    pub v: f32,
}

pub trait Geometry {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry>;

    fn intersect_p(&self, ray: &mut Ray) -> bool {
        self.intersect(ray).is_some()
    }
}

pub struct DifferentialGeometry<'a> {
    pub phit: Point,
    pub nhit: Vector,
    pub dpdu: Vector,
    pub dpdv: Vector,
    pub tex_coord: TextureCoordinate,
    pub geom: &'a Geometry,
}

impl<'a> DifferentialGeometry<'a> {
    pub fn new(p: Point,
               n: Vector,
               dpdu: Vector,
               dpdv: Vector,
               tex_coord: TextureCoordinate,
               geom: &Geometry)
               -> DifferentialGeometry {
        DifferentialGeometry {
            phit: p,
            nhit: n,
            dpdu: dpdu,
            dpdv: dpdv,
            tex_coord: tex_coord,
            geom: geom,
        }
    }

    pub fn transform(&mut self, transform: Transform, inv_transform: Transform) {
        self.phit = transform * self.phit;
        self.nhit = transform::transform_normal(&self.nhit, &inv_transform).normalize();
    }
}

pub trait BoundedGeometry: Geometry + Bounded {}
impl<T> BoundedGeometry for T where T: Geometry + Bounded {}
