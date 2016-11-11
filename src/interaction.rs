use ::{Point, Point2f, Vector, Transform};
use bsdf::BSDF;
use colour::Colourf;
use primitive::Primitive;
use shapes::Shape;
use transform;
use na::{self, Cross, Norm};

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

    pub fn le(&self, wo: Vector) -> Colourf {
        Colourf::black()
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
            bsdf: self.bsdf,
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
