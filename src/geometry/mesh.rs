extern crate tobj;

use std::path::Path;
use std::sync::Arc;

use Point;
use Vector;
use stats;
use geometry::{Geometry, TextureCoordinate, DifferentialGeometry};
use geometry::bbox::BBox;
use ray::Ray;
use na::{Vector2, Norm, Dot, Cross};

pub struct Mesh {
    pub tris: Vec<MeshTriangle>,
    bbox: BBox,
}

impl Mesh {
    pub fn load(file: &Path, model_name: &str) -> Mesh {
        let (models, _) = tobj::load_obj(file).unwrap();
        let model = models.iter()
            .find(|m| m.name == model_name)
            .unwrap();

        let positions: Arc<Vec<Point>> = Arc::new(model.mesh
            .positions
            .chunks(3)
            .map(|p| Point::new(p[0], p[1], p[2]))
            .collect());

        let normals: Arc<Vec<Vector>> = Arc::new(model.mesh
            .normals
            .chunks(3)
            .map(|n| Vector::new(n[0], n[1], n[2]))
            .collect());

        let uv: Arc<Vec<Vector2<f32>>> = Arc::new(model.mesh
            .texcoords
            .chunks(2)
            .map(|t| Vector2::new(t[0], t[1]))
            .collect());

        let tris = model.mesh
            .indices
            .chunks(3)
            .map(|i| {
                stats::inc_num_triangles();
                MeshTriangle {
                    a: i[0] as usize,
                    b: i[1] as usize,
                    c: i[2] as usize,
                    p: positions.clone(),
                    n: normals.clone(),
                    t: uv.clone(),
                }
            })
            .collect();

        let mut bbox = BBox::new();
        for p in &*positions {
            bbox.extend(p);
        }

        Mesh {
            tris: tris,
            bbox: bbox,
        }
    }
}

impl Geometry for Mesh {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        let mut result: Option<DifferentialGeometry> = None;

        if !self.bbox.intersect(ray) {
            return None;
        }

        for t in &self.tris {
            result = t.intersect(ray).or(result)
        }

        result
    }
}

pub struct MeshTriangle {
    a: usize,
    b: usize,
    c: usize,
    p: Arc<Vec<Point>>,
    n: Arc<Vec<Vector>>,
    t: Arc<Vec<Vector2<f32>>>,
}

impl Geometry for MeshTriangle {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        stats::inc_triangle_test();
        let v0 = self.p[self.a];
        let v1 = self.p[self.b];
        let v2 = self.p[self.c];
        let v0v1 = v1 - v0;
        let v0v2 = v2 - v0;
        let pvec = ray.dir.cross(&v0v2);
        let det = v0v1.dot(&pvec);

        if det.abs() < 1e-4 {
            return None;
        }

        let inv_det = 1.0 / det;
        let tvec = ray.origin - v0;
        let v = tvec.dot(&pvec) * inv_det;
        if v < 0.0 || v > 1.0 {
            return None;
        }

        let qvec = tvec.cross(&v0v1);
        let w = ray.dir.dot(&qvec) * inv_det;
        if w < 0.0 || v + w > 1.0 {
            return None;
        }

        let thit = v0v2.dot(&qvec) * inv_det;
        if thit < ray.t_min || thit > ray.t_max {
            return None;
        } else {
            ray.t_max = thit;
        }

        let u = 1.0 - v - w;
        let na = self.n[self.a];
        let nb = self.n[self.b];
        let nc = self.n[self.c];
        let nhit = (u * na + v * nb + w * nc).normalize();

        let ta = self.t[self.a];
        let tb = self.t[self.b];
        let tc = self.t[self.c];
        let uv = u * ta + v * tb + w * tc;
        let texcoord = TextureCoordinate { u: uv.x, v: uv.y };

        stats::inc_triangle_isect();
        Some(DifferentialGeometry::new(ray.at(ray.t_max), nhit, texcoord, self))
    }
}

#[test]
fn testObj() {
    let mesh = Mesh::load(&Path::new("models/cone.obj"), "Cone");

    assert_eq!(mesh.tris.is_empty(), false);
}
