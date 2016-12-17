use na::{Inverse, Cross, Norm};
use na::{abs, zero};

use ::{Transform, Point2f, Point3f, Vector3f, max_dimension, permute_v, permute_p,
       coordinate_system};
use bounds::Bounds3f;
use interaction::{Interaction, SurfaceInteraction};
use ray::Ray;
use shapes::Shape;

pub struct TriangleMesh {
    object_to_world: Transform,
    world_to_object: Transform,
    n_triangles: usize,
    n_vertices: usize,
    vertex_indices: Vec<usize>,
    p: Vec<Point3f>,
    n: Vec<Vector3f>,
    s: Vec<Vector3f>,
    uv: Option<Vec<Point2f>>, // TODO alpha mask
}

impl TriangleMesh {
    pub fn new(object_to_world: &Transform,
               n_triangles: usize,
               vertex_indices: &[usize],
               n_vertices: usize,
               p: &[Point3f],
               s: &[Vector3f],
               n: &[Vector3f],
               uv: &[Point2f])
               -> Self {
        let points: Vec<Point3f> = p.iter().map(|pt| *object_to_world * *pt).collect();
        TriangleMesh {
            object_to_world: *object_to_world,
            world_to_object: object_to_world.inverse().unwrap(),
            n_triangles: n_triangles,
            n_vertices: n_vertices,
            vertex_indices: Vec::from(vertex_indices),
            p: points,
            n: Vec::from(n),
            s: Vec::from(s),
            uv: Some(Vec::from(uv)),
        }
    }
}

pub struct Triangle<'a> {
    mesh: &'a TriangleMesh,
    v: &'a [usize],
}

impl<'a> Triangle<'a> {
    pub fn new(mesh: &'a TriangleMesh, tri_number: usize) -> Triangle<'a> {
        Triangle {
            mesh: mesh,
            v: &mesh.vertex_indices[tri_number * 3..tri_number * 3 + 1],
        }
    }

    fn get_uvs(&self) -> [Point2f; 3] {
        if let Some(ref uv) = self.mesh.uv {
            [uv[self.v[0]], uv[self.v[1]], uv[self.v[2]]]
        } else {
            [Point2f::new(0.0, 0.0), Point2f::new(1.0, 0.0), Point2f::new(1.0, 1.0)]
        }
    }
}

impl<'a> Shape for Triangle<'a> {
    fn intersect(&self, ray: &Ray) -> Option<(SurfaceInteraction, f32)> {
        let p0 = self.mesh.p[self.v[0]];
        let p1 = self.mesh.p[self.v[1]];
        let p2 = self.mesh.p[self.v[2]];

        // Perform ray-triangle intersection test
        // - transform triangle vertices to ray coordinate space
        // -- translate vertices based on ray origin
        let mut p0t = p0 - ray.o.to_vector();
        let mut p1t = p1 - ray.o.to_vector();
        let mut p2t = p2 - ray.o.to_vector();
        // -- permute components of triangle vertices and ray direction
        let kz = max_dimension(abs(&ray.d));
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
        if det < 0.0 && (t_scaled >= 0.0 || t_scaled < ray.t_max * det) {
            return None;
        } else if det > 0.0 && (t_scaled <= 0.0 || t_scaled > ray.t_max * det) {
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
        let p_error = zero();
        // interpolate (u,v) parametric coordinates and hit point
        let p_hit = (p0.to_vector() * b0 + p1.to_vector() * b1 + p2.to_vector() * b2).to_point();
        let uv_hit = (uv[0].to_vector() * b0 + uv[1].to_vector() * b1 + uv[2].to_vector() * b2)
            .to_point();
        // test intersection against alpha texture if present
        // TODO
        // Fill in SurfaceInteraction from triangle hit
        Some((SurfaceInteraction::new(p_hit, p_error, uv_hit, -ray.d, dpdu, dpdv, self), t))
    }

    fn area(&self) -> f32 {
        let p0 = self.mesh.p[self.v[0]];
        let p1 = self.mesh.p[self.v[1]];
        let p2 = self.mesh.p[self.v[2]];

        0.5 * (p1 - p0).cross(&(p2 - p0)).norm()
    }

    fn object_bounds(&self) -> Bounds3f {
        let p0 = self.mesh.world_to_object * self.mesh.p[self.v[0]];
        let p1 = self.mesh.world_to_object * self.mesh.p[self.v[1]];
        let p2 = self.mesh.world_to_object * self.mesh.p[self.v[2]];
        Bounds3f::union_point(&Bounds3f::from_points(&p0, &p1), &p2)
    }

    fn world_bounds(&self) -> Bounds3f {
        let p0 = self.mesh.p[self.v[0]];
        let p1 = self.mesh.p[self.v[1]];
        let p2 = self.mesh.p[self.v[2]];
        Bounds3f::union_point(&Bounds3f::from_points(&p0, &p1), &p2)
    }

    fn sample(&self, u: &Point2f) -> (Interaction, f32) {
        unimplemented!();
    }
}
