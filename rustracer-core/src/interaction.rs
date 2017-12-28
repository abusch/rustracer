use std::sync::Arc;

use light_arena::Allocator;
use num::zero;

use {Normal3f, Point2f, Point3f, Transform, Vector2f, Vector3f};
use bsdf::BSDF;
use geometry::{face_forward_n, offset_ray_origin};
use material::TransportMode;
use primitive::Primitive;
use ray::Ray;
use shapes::Shape;
use spectrum::Spectrum;
use transform;

#[derive(Copy, Clone)]
pub struct Interaction {
    /// The point where the ray hit the primitive
    pub p: Point3f,
    /// Error bound for the intersection point
    pub p_error: Vector3f,
    /// Outgoing direction of the light at the intersection point (usually `-ray.d`)
    pub wo: Vector3f,
    /// Normal
    pub n: Normal3f,
}

impl Interaction {
    pub fn empty() -> Self {
        Interaction {
            p: zero(),
            p_error: zero(),
            wo: zero(),
            n: zero(),
        }
    }

    pub fn new(p: Point3f, p_error: Vector3f, wo: Vector3f, n: Normal3f) -> Interaction {
        Interaction {
            p: p,
            p_error: p_error,
            wo: wo.normalize(),
            n: n,
        }
    }

    pub fn from_point(p: &Point3f) -> Interaction {
        Interaction {
            p: *p,
            p_error: zero(),
            wo: zero(),
            n: zero(),
        }
    }

    pub fn spawn_ray(&self, dir: &Vector3f) -> Ray {
        assert!(dir.x != 0.0 || dir.y != 0.0 || dir.z != 0.0);
        let o = offset_ray_origin(&self.p, &self.p_error, &self.n, dir);
        Ray::new(o, *dir)
    }

    pub fn spawn_ray_to(&self, p: &Point3f) -> Ray {
        let o = offset_ray_origin(&self.p, &self.p_error, &self.n, &(*p - self.p));
        let d = *p - self.p;
        assert!(d.x != 0.0 || d.y != 0.0 || d.z != 0.0);
        Ray::segment(o, d, 1.0 - 1e-4)
    }

    pub fn spawn_ray_to_interaction(&self, it: &Interaction) -> Ray {
        let origin = offset_ray_origin(&self.p, &self.p_error, &self.n, &(it.p - self.p));
        let target = offset_ray_origin(&it.p, &it.p_error, &it.n, &(origin - it.p));
        let d = target - origin;
        Ray::segment(origin, d, 1.0 - 1e-4)
    }
}

#[derive(Clone)]
pub struct SurfaceInteraction<'a, 'b> {
    pub hit: Interaction,
    /// Texture coordinates
    pub uv: Point2f,
    /// Partial derivatives at the intersection point
    pub dpdu: Vector3f,
    pub dpdv: Vector3f,
    /// Partial derivatives of the normal
    pub dndu: Normal3f,
    pub dndv: Normal3f,
    /// Ray differentials
    pub dpdx: Vector3f,
    pub dpdy: Vector3f,
    ///
    pub dudx: f32,
    pub dvdx: f32,
    pub dudy: f32,
    pub dvdy: f32,
    /// Hit shape
    pub shape: &'a Shape,
    /// Hit primitive
    pub primitive: Option<&'a Primitive>,
    /// Shading information
    pub shading: Shading,
    /// BSDF of the surface at the intersection point
    pub bsdf: Option<Arc<BSDF<'b>>>,
}

impl<'a, 'b> SurfaceInteraction<'a, 'b> {
    pub fn new(p: Point3f,
               p_error: Vector3f,
               uv: Point2f,
               wo: Vector3f,
               dpdu: Vector3f,
               dpdv: Vector3f,
               dndu: Normal3f,
               dndv: Normal3f,
               shape: &Shape)
               -> SurfaceInteraction {
        let mut n = Normal3f::from(dpdu.cross(&dpdv).normalize());
        if shape.reverse_orientation() ^ shape.transform_swaps_handedness() {
            n *= -1.0;
        }
        SurfaceInteraction {
            hit: Interaction::new(p, p_error, wo.normalize(), n),
            uv,
            dpdu,
            dpdv,
            dndu,
            dndv,
            dpdx: zero(),
            dpdy: zero(),
            dudx: 0.0,
            dvdx: 0.0,
            dudy: 0.0,
            dvdy: 0.0,
            shape: shape,
            primitive: None,
            // Initialize shading geometry from true geometry
            shading: Shading {
                n: n,
                dpdu: dpdu,
                dpdv: dpdv,
                dndu: dndu,
                dndv: dndv,
            },
            bsdf: None,
        }
    }

    pub fn le(&self, w: &Vector3f) -> Spectrum {
        self.primitive
            .and_then(|p| p.area_light())
            .map(|light| light.l(self.into(), w))
            .unwrap_or_else(Spectrum::black)
    }

    pub fn transform(&self, t: &Transform) -> SurfaceInteraction<'a, 'b> {
        let (p, p_err) = t.transform_point_with_error(&self.hit.p, &self.hit.p_error);
        let mut si = SurfaceInteraction {
            hit: Interaction::new(p,
                                  p_err,
                                  (t * &self.hit.wo).normalize(),
                                  t.transform_normal(&self.hit.n).normalize()),
            uv: self.uv,
            dpdu: t * &self.dpdu,
            dpdv: t * &self.dpdv,
            dndu: zero(),
            dndv: zero(),
            dpdx: zero(),
            dpdy: zero(),
            dudx: 0.0,
            dvdx: 0.0,
            dudy: 0.0,
            dvdy: 0.0,
            shape: self.shape,
            primitive: self.primitive,
            shading: Shading {
                n: t.transform_normal(&self.shading.n).normalize(),
                dpdu: t * &self.shading.dpdu,
                dpdv: t * &self.shading.dpdv,
                dndu: zero(),
                dndv: zero(),
            },
            bsdf: self.bsdf.clone(),
        };
        si.shading.n = face_forward_n(&si.shading.n, &si.hit.n);

        si
    }

    pub fn compute_scattering_functions(&mut self,
                                        ray: &Ray,
                                        transport: TransportMode,
                                        allow_multiple_lobes: bool,
                                        arena: &'b Allocator) {
        self.compute_differential(ray);
        if let Some(primitive) = self.primitive {
            primitive.compute_scattering_functions(self, transport, allow_multiple_lobes, arena);
        }
    }

    pub fn spawn_ray(&self, dir: &Vector3f) -> Ray {
        assert!(dir.x != 0.0 || dir.y != 0.0 || dir.z != 0.0);
        let o = offset_ray_origin(&self.hit.p, &self.hit.p_error, &self.hit.n, dir);
        Ray::new(o, *dir)
    }

    pub fn spawn_ray_to(&self, p: &Point3f) -> Ray {
        let d = *p - self.hit.p;
        assert!(d.x != 0.0 || d.y != 0.0 || d.z != 0.0);
        let o = offset_ray_origin(&self.hit.p, &self.hit.p_error, &self.hit.n, &d);
        Ray::segment(o, d, 1.0 - 1e-4)
    }

    pub fn set_shading_geometry(&mut self,
                                dpdus: &Vector3f,
                                dpdvs: &Vector3f,
                                dndus: &Normal3f,
                                dndvs: &Normal3f,
                                is_orientation_authoritative: bool) {
        // Compute shading.n for SurfaceInteraction
        self.shading.n = Normal3f::from(dpdus.cross(dpdvs).normalize());
        if self.shape.reverse_orientation() ^ self.shape.transform_swaps_handedness() {
            self.shading.n *= -1.0;
        }
        if is_orientation_authoritative {
            self.hit.n = face_forward_n(&self.hit.n, &self.shading.n);
        } else {
            self.shading.n = face_forward_n(&self.shading.n, &self.hit.n);
        }

        // Initialize shading partial derivative values
        self.shading.dpdu = *dpdus;
        self.shading.dpdv = *dpdvs;
        self.shading.dndu = *dndus;
        self.shading.dndv = *dndvs;
    }

    #[allow(non_snake_case)]
    pub fn compute_differential(&mut self, ray: &Ray) {
        if let Some(ref diff) = ray.differential {
            // Estimate screen space change in p and (u,v)

            // Compute auxiliary intersection points with plane
            let d = self.hit
                .n
                .dot(&Vector3f::new(self.hit.p.x, self.hit.p.y, self.hit.p.z));
            let tx = -(self.hit.n.dot(&Vector3f::from(diff.rx_origin)) - d) /
                     self.hit.n.dot(&diff.rx_direction);
            let ty = -(self.hit.n.dot(&Vector3f::from(diff.ry_origin)) - d) /
                     self.hit.n.dot(&diff.ry_direction);
            if tx.is_infinite() || tx.is_nan() || ty.is_infinite() || ty.is_nan() {
                self.dudx = 0.0;
                self.dudy = 0.0;
                self.dvdx = 0.0;
                self.dvdy = 0.0;
                self.dpdx = zero();
                self.dpdy = zero();
                return;
            }

            let px = diff.rx_origin + tx * diff.rx_direction;
            let py = diff.ry_origin + ty * diff.ry_direction;
            self.dpdx = px - self.hit.p;
            self.dpdy = py - self.hit.p;
            // Compute (u,v) offsets at auxiliary points

            // Choose two dimensions to use for ray offset computation
            let mut dim: [usize; 2] = [0; 2];
            if self.hit.n.x.abs() > self.hit.n.y.abs() && self.hit.n.x.abs() > self.hit.n.z.abs() {
                dim[0] = 1;
                dim[1] = 2;
            } else if self.hit.n.y.abs() > self.hit.n.z.abs() {
                dim[0] = 0;
                dim[1] = 2;
            } else {
                dim[0] = 0;
                dim[1] = 1;
            }
            // Initialize A, Bx, and By matrices for offset computation
            let A = [[self.dpdu[dim[0]], self.dpdv[dim[0]]],
                     [self.dpdu[dim[1]], self.dpdv[dim[1]]]];
            let Bx = Vector2f::new(px[dim[0]] - self.hit.p[dim[0]],
                                   px[dim[1]] - self.hit.p[dim[1]]);
            let By = Vector2f::new(py[dim[0]] - self.hit.p[dim[0]],
                                   py[dim[1]] - self.hit.p[dim[1]]);


            let (dudx, dvdx) = transform::solve_linear_system2x2(&A, &Bx).unwrap_or((0.0, 0.0));
            let (dudy, dvdy) = transform::solve_linear_system2x2(&A, &By).unwrap_or((0.0, 0.0));
            self.dudx = dudx;
            self.dvdx = dvdx;
            self.dudy = dudy;
            self.dvdy = dvdy;
        } else {
            self.dudx = 0.0;
            self.dudy = 0.0;
            self.dvdx = 0.0;
            self.dvdy = 0.0;
            self.dpdx = zero();
            self.dpdy = zero();
        }
    }
}

impl<'a, 'b> From<SurfaceInteraction<'a, 'b>> for Interaction {
    fn from(si: SurfaceInteraction) -> Interaction {
        si.hit
    }
}

impl<'a, 'b> From<&'a SurfaceInteraction<'a, 'b>> for &'a Interaction {
    fn from(si: &'a SurfaceInteraction<'a, 'b>) -> &'a Interaction {
        &si.hit
    }
}

/// Normal and partial derivatives used for shading. Can be different from geometric ones due to
/// bump mapping, etc.
#[derive(Clone)]
pub struct Shading {
    pub n: Normal3f,
    pub dpdu: Vector3f,
    pub dpdv: Vector3f,
    pub dndu: Normal3f,
    pub dndv: Normal3f,
}

impl Default for Shading {
    fn default() -> Self {
        Shading {
            n: zero(),
            dpdu: zero(),
            dpdv: zero(),
            dndu: zero(),
            dndv: zero(),
        }
    }
}
