use {Vector, Point, Transform};
use ray::Ray;
use na::{Norm, Dot, ToHomogeneous, FromHomogeneous, Transpose, Matrix3, Matrix4, Inverse};

pub use self::bbox::*;
pub use self::sphere::*;
pub use self::plane::*;
pub use self::triangle::*;
pub use self::mesh::*;

mod sphere;
mod plane;
mod triangle;
mod mesh;
mod bbox;

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

    pub fn transform(&mut self, transform: Transform, inv_transform: Transform) {
        self.phit = transform * self.phit;
        self.nhit = transform_normal(&self.nhit, &inv_transform).normalize();
    }
}

fn transform_normal(normal: &Vector, transform: &Transform) -> Vector {
    let hom: Matrix4<f32> = transform.to_homogeneous();
    let m: Matrix3<f32> = FromHomogeneous::from(&hom);
    let m_transp = m.transpose();
    *normal * m_transp
}

pub trait BoundedGeometry: Geometry + Bounded {}
impl<T> BoundedGeometry for T where T: Geometry + Bounded {}

#[test]
fn test_normal_transform() {
    let t = Transform::new(Vector::new(0.0, 0.0, 0.0), Vector::new(4.0, 5.0, 6.0), 4.0);
    let t_inv = t.inverse().unwrap();

    let v = Vector::x();
    let n = Vector::y();
    println!("v = {}, n = {}", v, n);
    assert_eq!(v.dot(&n), 0.0);

    let v2 = t * v;
    let n2 = transform_normal(&n, &t_inv);
    println!("v = {}, n = {}", v2, n2);
    relative_eq!(v2.dot(&n2), 0.0);
}
