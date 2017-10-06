extern crate tobj;

use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;

use na::{Point2, Point3};

use {Transform, Point2f, Point3f, Vector3f, max_dimension, permute_v, permute_p,
     coordinate_system, gamma};
use bounds::Bounds3f;
use interaction::{Interaction, SurfaceInteraction};
use paramset::ParamSet;
use ray::Ray;
use shapes::Shape;
use stats;
use texture::Texture;

pub struct TriangleMesh {
    world_to_object: Transform,
    vertex_indices: Vec<usize>,
    p: Vec<Point3f>,
    n: Option<Vec<Vector3f>>,
    s: Option<Vec<Vector3f>>,
    uv: Option<Vec<Point2f>>, // TODO alpha mask
}

impl TriangleMesh {
    pub fn new(object_to_world: &Transform,
               vertex_indices: &[usize],
               p: &[Point3f],
               s: Option<&[Vector3f]>,
               n: Option<&[Vector3f]>,
               uv: Option<&[Point2f]>)
               -> Self {
        let points: Vec<Point3f> = p.iter().map(|pt| object_to_world * pt).collect();
        TriangleMesh {
            world_to_object: object_to_world.inverse(),
            vertex_indices: Vec::from(vertex_indices),
            p: points,
            n: n.map(Vec::from),
            s: s.map(Vec::from),
            uv: uv.map(Vec::from),
        }
    }

    pub fn create(o2w: &Transform, w2o: &Transform, reverse_orientation: bool, params: &mut ParamSet, float_textures: &HashMap<String, Arc<Texture<f32> + Send + Sync>>) -> Vec<Arc<Shape + Send + Sync>> {
        let vi: Vec<usize> = params.find_int("indices").unwrap_or(Vec::new()).iter().map(|i| *i as usize).collect();
        let P = params.find_point3f("P").unwrap_or(Vec::new());
        let uvs = params
            .find_point2f("uv")
            .or(params.find_point2f("st"))
            .or(params
                .find_float("uv")
                .or(params.find_float("st")).map(|fuv| {
               fuv
               .chunks(2)
               .map(|s| Point2f::new(s[0], s[1]))
               .collect()}));
        // if !uvs.is_empty() {
        //     if uvs.len() < P.len() {
        //         error!("Not enough of \"uv\"s for triangle mesh. Expected {}, found {}. Discarding", P.len(), uvs.len());
        //         uvs.clear();
        //     } else if uvs.len() > P.len() {
        //         warn!("More \"uv\"s provided than will be used for triangle mesh. ({} expected, {} found)", P.len(), uvs.len());
        //     }
        // }
        if vi.is_empty() {
            error!("Vertex indices \"indices\" not provided with triangle mesh shape");
            return Vec::new();
        }
        if P.is_empty() {
            error!("Vertex positions \"P\" not provided with triangle mesh shape");
            return Vec::new();
        }
        let S = params.find_vector3f("S").and_then(|s| {
            if s.len() != P.len() {
                error!("Number of \"S\"s for mesh triangle must match \"P\"s");
                None
            } else {
                Some(s)
            }
        });
        // TODO should be Normal3f
        let N = params.find_vector3f("N").and_then(|n| {
            if n.len() != P.len() {
                error!("Number of \"N\"s for mesh triangle must match \"P\"s");
                None
            } else {
                Some(n)
            }
        });

        // TODO implement rest of the validation / sanity checking

        let res: Vec<Arc<Shape + Send + Sync>> = create_triangle_mesh(
            o2w,
             reverse_orientation,
              &vi[..],
               &P[..],
                S.as_ref().map(|s| &s[..]),
                 N.as_ref().map(|n| &n[..]),
                  uvs.as_ref().map(|uv| &uv[..]));

                    res
    }
}

pub struct Triangle {
    mesh: Arc<TriangleMesh>,
    v_start_index: usize,
}

impl Triangle {
    pub fn new(mesh: Arc<TriangleMesh>, tri_number: usize) -> Triangle {
        Triangle {
            mesh: mesh,
            v_start_index: tri_number * 3,
        }
    }

    #[inline(always)]
    fn v(&self, index: usize) -> usize {
        self.mesh.vertex_indices[self.v_start_index + index]
    }

    fn get_uvs(&self) -> [Point2f; 3] {
        if let Some(ref uv) = self.mesh.uv {
            [uv[self.v(0)], uv[self.v(1)], uv[self.v(2)]]
        } else {
            [Point2f::new(0.0, 0.0),
             Point2f::new(1.0, 0.0),
             Point2f::new(1.0, 1.0)]
        }
    }
}

impl Shape for Triangle {
    fn intersect(&self, ray: &Ray) -> Option<(SurfaceInteraction, f32)> {
        stats::inc_triangle_test();
        let p0 = self.mesh.p[self.v(0)];
        let p1 = self.mesh.p[self.v(1)];
        let p2 = self.mesh.p[self.v(2)];

        // Perform ray-triangle intersection test
        // - transform triangle vertices to ray coordinate space
        // -- translate vertices based on ray origin
        let mut p0t = p0 - ray.o.coords;
        let mut p1t = p1 - ray.o.coords;
        let mut p2t = p2 - ray.o.coords;
        // -- permute components of triangle vertices and ray direction
        let kz = max_dimension(ray.d.abs());
        let mut kx = kz + 1;
        if kx == 3 {
            kx = 0;
        }
        let mut ky = kx + 1;
        if ky == 3 {
            ky = 0;
        }
        let d = permute_v(&ray.d, kx, ky, kz);
        p0t = permute_p(&p0t, kx, ky, kz);
        p1t = permute_p(&p1t, kx, ky, kz);
        p2t = permute_p(&p2t, kx, ky, kz);
        // -- apply shear transformation to translated vertex positions
        let sx = -d.x / d.z;
        let sy = -d.y / d.z;
        let sz = 1.0 / d.z;
        p0t.x += sx * p0t.z;
        p0t.y += sy * p0t.z;
        p1t.x += sx * p1t.z;
        p1t.y += sy * p1t.z;
        p2t.x += sx * p2t.z;
        p2t.y += sy * p2t.z;
        // - compute edge function coefficients
        let e0 = p1t.x * p2t.y - p1t.y * p2t.x;
        let e1 = p2t.x * p0t.y - p2t.y * p0t.x;
        let e2 = p0t.x * p1t.y - p0t.y * p1t.x;
        // - fall back to double precision at edges
        // TODO
        // - perform triangle edge and determinant test
        if (e0 < 0.0 || e1 < 0.0 || e2 < 0.0) && (e0 > 0.0 || e1 > 0.0 || e2 > 0.0) {
            return None;
        }
        let det = e0 + e1 + e2;
        if det == 0.0 {
            return None;
        }
        // - compute scaled hit distance to triangle and test against ray t range
        p0t.z *= sz;
        p1t.z *= sz;
        p2t.z *= sz;
        let t_scaled = e0 * p0t.z + e1 * p1t.z + e2 * p2t.z;
        if (det < 0.0 && (t_scaled >= 0.0 || t_scaled < ray.t_max * det)) ||
           (det > 0.0 && (t_scaled <= 0.0 || t_scaled > ray.t_max * det)) {
            return None;
        }
        // - compute barycentric coordinates and t value for triangle intersection
        let inv_det = 1.0 / det;
        let b0 = e0 * inv_det;
        let b1 = e1 * inv_det;
        let b2 = e2 * inv_det;
        let t = t_scaled * inv_det;
        // - ensure that computed triangle t is conservatively greater than zero
        // Compute triangle partial derivatives
        let uv = self.get_uvs();
        // - compute deltas for partial derivatives
        let duv02 = uv[0] - uv[2];
        let duv12 = uv[1] - uv[2];
        let dp02 = p0 - p2;
        let dp12 = p1 - p2;
        let determinant = duv02[0] * duv12[1] - duv02[1] * duv12[0];
        let (dpdu, dpdv) = if determinant == 0.0 {
            // handle zero determinant
            coordinate_system(&(p2 - p0).cross(&(p1 - p0)).normalize())
        } else {
            let inv_det = 1.0 / determinant;
            let dpdu = (duv12[1] * dp02 - duv02[1] * dp12) / inv_det;
            let dpdv = (-duv12[0] * dp02 + duv02[0] * dp12) / inv_det;
            (dpdu, dpdv)
        };
        // Compute error bounds for triangle intersection
        let x_abs_sum = (b0 * p0.x).abs() + (b1 * p1.x).abs() + (b2 * p2.x).abs();
        let y_abs_sum = (b0 * p0.y).abs() + (b1 * p1.y).abs() + (b2 * p2.y).abs();
        let z_abs_sum = (b0 * p0.z).abs() + (b1 * p1.z).abs() + (b2 * p2.z).abs();
        let p_error = gamma(7) * Vector3f::new(x_abs_sum, y_abs_sum, z_abs_sum);
        // interpolate (u,v) parametric coordinates and hit point
        let p_hit = Point3::from_coordinates(p0.coords * b0 + p1.coords * b1 + p2.coords * b2);
        let uv_hit = Point2::from_coordinates(uv[0].coords * b0 + uv[1].coords * b1 +
                                              uv[2].coords * b2);
        // test intersection against alpha texture if present
        // TODO
        // Fill in SurfaceInteraction from triangle hit
        let mut isect = SurfaceInteraction::new(p_hit, p_error, uv_hit, -ray.d, dpdu, dpdv, self);
        // - Override surface normal
        let n = dp02.cross(&dp12).normalize();
        isect.n = n;
        isect.shading.n = n;
        // Initialize triangle shading geometry
        // - shading normal
        let ns = if let Some(ref n) = self.mesh.n {
            (n[self.v(0)] * b0 + n[self.v(1)] * b1 + n[self.v(2)] * b2).normalize()
        } else {
            isect.n
        };
        // - shading tangent
        let mut ss = if let Some(ref s) = self.mesh.s {
            (s[self.v(0)] * b0 + s[self.v(1)] * b1 + s[self.v(2)] * b2).normalize()
        } else {
            isect.dpdu.normalize()
        };
        // - shading bitangent
        let mut ts = ss.cross(&ns);
        if ts.norm_squared() > 0.0 {
            ts = ts.normalize();
            // adjust ss to make sure it's orthogonal with ns and ts
            ss = ts.cross(&ns);
        } else {
            let (ss1, ts1) = coordinate_system(&ns);
            ss = ss1;
            ts = ts1;
        }
        isect.shading.n = ns;
        isect.shading.dpdu = ss;
        isect.shading.dpdv = ts;
        // Ensure correct orientation of the geometric normal TODO

        stats::inc_triangle_isect();
        Some((isect, t))
    }

    fn area(&self) -> f32 {
        let p0 = self.mesh.p[self.v(0)];
        let p1 = self.mesh.p[self.v(1)];
        let p2 = self.mesh.p[self.v(2)];

        0.5 * (p1 - p0).cross(&(p2 - p0)).norm()
    }

    fn object_bounds(&self) -> Bounds3f {
        let p0 = &self.mesh.world_to_object * &self.mesh.p[self.v(0)];
        let p1 = &self.mesh.world_to_object * &self.mesh.p[self.v(1)];
        let p2 = &self.mesh.world_to_object * &self.mesh.p[self.v(2)];
        Bounds3f::union_point(&Bounds3f::from_points(&p0, &p1), &p2)
    }

    fn world_bounds(&self) -> Bounds3f {
        let p0 = self.mesh.p[self.v(0)];
        let p1 = self.mesh.p[self.v(1)];
        let p2 = self.mesh.p[self.v(2)];
        Bounds3f::union_point(&Bounds3f::from_points(&p0, &p1), &p2)
    }

    fn sample(&self, _u: &Point2f) -> (Interaction, f32) {
        unimplemented!();
    }
}

pub fn create_square(object_to_world: &Transform, reverse_orientation: bool) -> Vec<Arc<Shape + Send + Sync>> {
    let vertices = vec![Point3f::new(-0.5, -0.5, 0.0),
                        Point3f::new(-0.5, 0.5, 0.0),
                        Point3f::new(0.5, 0.5, 0.0),
                        Point3f::new(0.5, -0.5, 0.0)];
    let indices = vec![0, 1, 2, 0, 2, 3];
    let uv = vec![Point2f::new(0.0, 0.0),
                  Point2f::new(0.0, 1.0),
                  Point2f::new(1.0, 1.0),
                  Point2f::new(1.0, 0.0)];

    create_triangle_mesh(object_to_world,
                         reverse_orientation,
                         &indices[..],
                         &vertices[..],
                         None,
                         None,
                         Some(&uv[..]))
}

pub fn create_triangle_mesh(object_to_world: &Transform,
                            _reverse_orientation: bool,
                            vertex_indices: &[usize],
                            p: &[Point3f],
                            s: Option<&[Vector3f]>,
                            n: Option<&[Vector3f]>,
                            uv: Option<&[Point2f]>)
                            -> Vec<Arc<Shape + Send + Sync>> {
    let mesh = Arc::new(TriangleMesh::new(object_to_world, vertex_indices, p, s, n, uv));

    let n_triangles = vertex_indices.len() / 3;
    let mut tris: Vec<Arc<Shape + Send + Sync>> = Vec::with_capacity(n_triangles);

    for i in 0..n_triangles {
        stats::inc_num_triangles();
        tris.push(Arc::new(Triangle::new(mesh.clone(), i)));
    }

    tris
}

pub fn load_triangle_mesh(file: &Path, model_name: &str, transform: &Transform) -> Vec<Arc<Shape + Send + Sync>> {
    info!("Loading {} model from OBJ file:", model_name);
    let (models, _) = tobj::load_obj(file.into()).unwrap();
    let model = models.iter().find(|m| m.name == model_name).unwrap();

    info!("\tProcessing indices");
    let indices: Vec<usize> = model
        .mesh
        .indices
        .iter()
        .map(|i| *i as usize)
        .collect();

    info!("\tProcessing vertices");
    let positions: Vec<Point3f> = model
        .mesh
        .positions
        .chunks(3)
        .map(|p| Point3f::new(p[0], p[1], p[2]))
        .collect();

    info!("\tProcessing normals");
    let normals: Vec<Vector3f> = model
        .mesh
        .normals
        .chunks(3)
        .map(|n| Vector3f::new(n[0], n[1], n[2]))
        .collect();

    info!("\tProcessing UV coordinates");
    let uv: Vec<Point2f> = model
        .mesh
        .texcoords
        .chunks(2)
        .map(|t| Point2f::new(t[0], t[1]))
        .collect();

    create_triangle_mesh(transform,
                         false,
                         &indices[..],
                         &positions[..],
                         None,
                         if normals.is_empty() {
                             None
                         } else {
                             Some(&normals[..])
                         },
                         if uv.is_empty() { None } else { Some(&uv[..]) })
}
