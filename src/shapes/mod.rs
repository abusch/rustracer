use std::sync::Arc;

use ::{Vector, Point, Point2f};
use bsdf::BSDF;
use ray::Ray;
use bounds::Bounds3f;
use primitive::Primitive;
use na::{self, Cross, Norm};

pub mod sphere;

pub struct SurfaceInteraction<'a> {
    /// The point where the ray hit the primitive
    pub p: Point,
    /// Error bound for the intersection point
    pub p_error: Vector,
    /// Outgoing direction of the light at the intersection point (usually `-ray.d`)
    pub wo: Vector,
    /// Normal
    pub n: Vector,
    /// Texture coordinates
    pub uv: Point2f,
    /// Partial derivatives at the intersection point
    pub dpdu: Vector,
    pub dpdv: Vector,
    /// Partial derivaties of the normal
    pub dndu: Vector,
    pub dndv: Vector,
    /// Hit shape
    pub shape: &'a Shape,
    /// Hit primitive
    pub primitive: Option<&'a Primitive>,
    /// Shading information
    pub shading: Shading,
    /// BSDF of the surface at the intersection point
    pub bsdf: Option<BSDF>,
}

impl<'a> SurfaceInteraction<'a> {
    pub fn new(p: Point,
               p_error: Vector,
               uv: Point2f,
               wo: Vector,
               dpdu: Vector,
               dpdv: Vector,
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
}

/// Normal and partial derivatives used for shading. Can be different from geometric ones due to
/// bump mapping, etc.
pub struct Shading {
    pub n: Vector,
    pub dpdu: Vector,
    pub dpdv: Vector,
    pub dndu: Vector,
    pub dndv: Vector,
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

pub trait Shape {
    fn intersect(&self, ray: &Ray) -> Option<(SurfaceInteraction, f32)>;

    fn intersect_p(&self, ray: &Ray) -> bool {
        self.intersect(ray).is_some()
    }

    fn area(&self) -> f32 {
        0.0
    }

    fn object_bounds(&self) -> Bounds3f;

    fn world_bounds(&self) -> Bounds3f;
}
