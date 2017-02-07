use std::sync::Arc;

use fp::Ieee754;
use na::{self, Dot, Cross, Norm};

use {Point3f, Point2f, Vector3f, Transform};
use bsdf::BSDF;
use material::TransportMode;
use primitive::Primitive;
use ray::Ray;
use shapes::Shape;
use spectrum::Spectrum;
use stats;
use transform;


// TODO Find a better design for this mess of inheritance...

#[derive(Copy, Clone)]
pub struct Interaction {
    /// The point where the ray hit the primitive
    pub p: Point3f,
    /// Error bound for the intersection point
    pub p_error: Vector3f,
    /// Outgoing direction of the light at the intersection point (usually `-ray.d`)
    pub wo: Vector3f,
    /// Normal
    pub n: Vector3f,
}

impl Interaction {
    pub fn new(p: Point3f, p_error: Vector3f, wo: Vector3f, n: Vector3f) -> Interaction {
        Interaction {
            p: p,
            p_error: p_error,
            wo: wo,
            n: n,
        }
    }

    pub fn from_point(p: &Point3f) -> Interaction {
        Interaction {
            p: *p,
            p_error: Vector3f::new(0.0, 0.0, 0.0),
            wo: Vector3f::new(0.0, 0.0, 0.0),
            n: Vector3f::new(0.0, 0.0, 0.0),
        }
    }

    pub fn spawn_ray(&self, dir: &Vector3f) -> Ray {
        assert!(dir.x != 0.0 && dir.y != 0.0 && dir.z != 0.0);
        stats::inc_secondary_ray();
        let o = offset_origin(&self.p, &self.p_error, &self.n, dir);
        Ray::new(o, *dir)
    }

    pub fn spawn_ray_to(&self, p: &Point3f) -> Ray {
        let d = *p - self.p;
        assert!(d.x != 0.0 && d.y != 0.0 && d.z != 0.0);
        stats::inc_secondary_ray();
        let o = offset_origin(&self.p, &self.p_error, &self.n, &d);
        Ray::segment(o, d, 1.0 - 1e-4)
    }

    pub fn spawn_ray_to_interaction(&self, it: &Interaction) -> Ray {
        stats::inc_secondary_ray();
        let origin = offset_origin(&self.p, &self.p_error, &self.n, &(it.p - self.p));
        let target = offset_origin(&it.p, &it.p_error, &it.n, &(origin - it.p));
        let d = target - origin;
        Ray::segment(origin, d, 1.0 - 1e-4)
    }
}

pub struct SurfaceInteraction<'a> {
    /// The point where the ray hit the primitive
    pub p: Point3f,
    /// Error bound for the intersection point
    pub p_error: Vector3f,
    /// Outgoing direction of the light at the intersection point (usually `-ray.d`)
    pub wo: Vector3f,
    /// Normal
    pub n: Vector3f,
    /// Texture coordinates
    pub uv: Point2f,
    /// Partial derivatives at the intersection point
    pub dpdu: Vector3f,
    pub dpdv: Vector3f,
    /// Partial derivatives of the normal
    pub dndu: Vector3f,
    pub dndv: Vector3f,
    /// Hit shape
    pub shape: &'a Shape,
    /// Hit primitive
    pub primitive: Option<&'a Primitive>,
    /// Shading information
    pub shading: Shading,
    /// BSDF of the surface at the intersection point
    pub bsdf: Option<Arc<BSDF>>,
}

impl<'a> SurfaceInteraction<'a> {
    pub fn new(p: Point3f,
               p_error: Vector3f,
               uv: Point2f,
               wo: Vector3f,
               dpdu: Vector3f,
               dpdv: Vector3f,
               shape: &Shape)
               -> SurfaceInteraction {
        let n = dpdu.cross(&dpdv).normalize();
        // TODO adjust normal for handedness
        SurfaceInteraction {
            p: p,
            p_error: p_error,
            n: n,
            uv: uv,
            wo: wo,
            dpdu: dpdu,
            dpdv: dpdv,
            dndu: na::zero(),
            dndv: na::zero(),
            shape: shape,
            primitive: None,
            // Initialize shading geometry from true geometry
            shading: Shading {
                n: n,
                dpdu: dpdu,
                dpdv: dpdv,
                dndu: na::zero(),
                dndv: na::zero(),
            },
            bsdf: None,
        }
    }

    pub fn le(&self, w: &Vector3f) -> Spectrum {
        self.primitive
            .and_then(|p| p.area_light())
            .map(|light| light.l(&self.into(), w))
            .unwrap_or_else(Spectrum::black)
    }

    pub fn transform(&self, t: &Transform) -> SurfaceInteraction<'a> {
        let (p, p_err) = transform::transform_point_with_error(t, &self.p, &self.p_error);
        SurfaceInteraction {
            p: p,
            p_error: p_err,
            n: transform::transform_normal(&self.n, t),
            uv: self.uv,
            wo: *t * self.wo,
            dpdu: *t * self.dpdu,
            dpdv: *t * self.dpdv,
            dndu: na::zero(),
            dndv: na::zero(),
            shape: self.shape,
            primitive: self.primitive,
            shading: Shading {
                n: transform::transform_normal(&self.n, t),
                dpdu: *t * self.dpdu,
                dpdv: *t * self.dpdv,
                dndu: na::zero(),
                dndv: na::zero(),
            },
            bsdf: self.bsdf.clone(),
        }
    }

    pub fn compute_scattering_functions(&mut self,
                                        _ray: &Ray,
                                        transport: TransportMode,
                                        allow_multiple_lobes: bool) {
        if let Some(primitive) = self.primitive {
            primitive.compute_scattering_functions(self, transport, allow_multiple_lobes);
        }
    }

    pub fn spawn_ray(&self, dir: &Vector3f) -> Ray {
        assert!(dir.x != 0.0 || dir.y != 0.0 || dir.z != 0.0);
        stats::inc_secondary_ray();
        let o = offset_origin(&self.p, &self.p_error, &self.n, dir);
        Ray::new(o, *dir)
    }

    pub fn spawn_ray_to(&self, p: &Point3f) -> Ray {
        let d = *p - self.p;
        assert!(d.x != 0.0 || d.y != 0.0 || d.z != 0.0);
        stats::inc_secondary_ray();
        let o = offset_origin(&self.p, &self.p_error, &self.n, &d);
        Ray::segment(o, d, 1.0 - 1e-4)
    }
}

fn offset_origin(p: &Point3f, p_err: &Vector3f, n: &Vector3f, w: &Vector3f) -> Point3f {
    let d = na::abs(n).dot(p_err);
    let mut offset = d * *n;
    if w.dot(n) < 0.0 {
        offset = -offset;
    }
    let mut po = *p + offset;
    for i in 0..3 {
        if offset[i] > 0.0 {
            po[i] = po[i].next();
        } else if offset[i] < 0.0 {
            po[i] = po[i].prev();
        }
    }
    po
}

impl<'a> From<SurfaceInteraction<'a>> for Interaction {
    fn from(si: SurfaceInteraction) -> Interaction {
        Interaction::new(si.p, si.p_error, si.wo, si.n)
    }
}

impl<'a> From<&'a SurfaceInteraction<'a>> for Interaction {
    fn from(si: &SurfaceInteraction) -> Interaction {
        Interaction::new(si.p, si.p_error, si.wo, si.n)
    }
}

/// Normal and partial derivatives used for shading. Can be different from geometric ones due to
/// bump mapping, etc.
pub struct Shading {
    pub n: Vector3f,
    pub dpdu: Vector3f,
    pub dpdv: Vector3f,
    pub dndu: Vector3f,
    pub dndv: Vector3f,
}

impl Default for Shading {
    fn default() -> Self {
        Shading {
            n: na::zero(),
            dpdu: na::zero(),
            dpdv: na::zero(),
            dndu: na::zero(),
            dndv: na::zero(),
        }
    }
}
