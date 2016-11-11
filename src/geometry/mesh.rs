extern crate tobj;

use std::path::Path;
use std::sync::Arc;

use Point;
use Vector;
use stats;
use bvh::BVH;
use geometry::{Geometry, TextureCoordinate, DifferentialGeometry};
use geometry::bbox::{BBox, Bounded};
use ray::Ray;
use na::{Vector2, Norm, Dot, Cross};

pub struct Mesh {
    bvh: BVH<MeshTriangle>,
    pub bbox: BBox,
}

impl Mesh {
    pub fn load(file: &Path, model_name: &str) -> Mesh {
        println!("Loading {} model from OBJ file:", model_name);
        let (models, _) = tobj::load_obj(file).unwrap();
        let model = models.iter()
            .find(|m| m.name == model_name)
            .unwrap();

        println!("\tProcessing vertices");
        let positions: Arc<Vec<Point>> = Arc::new(model.mesh
            .positions
            .chunks(3)
            .map(|p| Point::new(p[0], p[1], p[2]))
            .collect());

        println!("\tProcessing normals");
        let normals: Arc<Vec<Vector>> = Arc::new(model.mesh
            .normals
            .chunks(3)
            .map(|n| Vector::new(n[0], n[1], n[2]))
            .collect());

        println!("\tProcessing UV coordinates");
        let uv: Arc<Vec<Vector2<f32>>> = Arc::new(model.mesh
            .texcoords
            .chunks(2)
            .map(|t| Vector2::new(t[0], t[1]))
            .collect());

        println!("\tBuilding triangles");
        let mut tris = model.mesh
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
            bbox.extend(*p);
        }

        Mesh {
            bvh: BVH::new(16, &mut tris),
            bbox: bbox,
        }
    }
}

impl Geometry for Mesh {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        self.bvh.intersect(ray, |r, tri| tri.intersect(r))
    }
}

impl Bounded for Mesh {
    fn get_world_bounds(&self) -> BBox {
        self.bbox
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

impl MeshTriangle {
    ///  Moller-Trumbore intersection algorithm
    ///  See http://www.cs.virginia.edu/~gfx/Courses/2003/ImageSynthesis/papers/Acceleration/Fast%20MinimumStorage%20RayTriangle%20Intersection.pdf
    fn intersect_mt(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        stats::inc_triangle_test();
        let v0 = self.p[self.a];
        let v1 = self.p[self.b];
        let v2 = self.p[self.c];
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let pvec = ray.d.cross(&edge2);
        let det = edge1.dot(&pvec);

        if det.abs() == 0.0 {
            return None;
        }
        let inv_det = 1.0 / det;

        let tvec = ray.o - v0;
        let u = tvec.dot(&pvec) * inv_det;
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let qvec = tvec.cross(&edge1);
        let v = ray.d.dot(&qvec) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = edge2.dot(&qvec) * inv_det;

        if t < ray.t_min || t > ray.t_max {
            return None;
        } else {
            ray.t_max = t;
        }

        let w = 1.0 - u - v;
        let na = self.n[self.a];
        let nb = self.n[self.b];
        let nc = self.n[self.c];
        let nhit = (w * na + u * nb + v * nc).normalize();

        let ta = self.t[self.a];
        let tb = self.t[self.b];
        let tc = self.t[self.c];
        let uv = w * ta + u * tb + v * tc;
        let texcoord = TextureCoordinate { u: uv.x, v: uv.y };

        // Compute partial derivative
        let duv02 = ta - tc;
        let duv12 = ta - tb;
        let dp02 = v0 - v2;
        let dp12 = v1 - v2;
        let determinant = duv02.dot(&duv12);
        let (dpdu, dpdv) = if determinant == 0.0 {
            coordinate_system(&edge2.cross(&edge1).normalize())
        } else {
            let inv_determinant = 1.0 / determinant;
            let dpdu = (duv12.y * dp02 - duv02.y * dp12) * inv_determinant;
            let dpdv = (-duv12.x * dp02 + duv02.x * dp12) * inv_determinant;
            (dpdu, dpdv)
        };

        stats::inc_triangle_isect();
        Some(DifferentialGeometry::new(ray.at(ray.t_max), nhit, dpdu, dpdv, texcoord, self))
    }
}

fn coordinate_system(v1: &Vector) -> (Vector, Vector) {
    let v2 = if v1.x.abs() > v1.y.abs() {
        Vector::new(-v1.z, 0.0, v1.x) / (v1.x * v1.x + v1.z * v1.z).sqrt()
    } else {
        Vector::new(0.0, v1.z, -v1.y) / (v1.y * v1.y + v1.z * v1.z).sqrt()
    };
    let v3 = v1.cross(&v2);
    (v2, v3)
}

impl Geometry for MeshTriangle {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        self.intersect_mt(ray)
    }
}

impl Bounded for MeshTriangle {
    fn get_world_bounds(&self) -> BBox {
        let mut bbox = BBox::new();
        bbox.extend(self.p[self.a]);
        bbox.extend(self.p[self.b]);
        bbox.extend(self.p[self.c]);

        bbox
    }
}
