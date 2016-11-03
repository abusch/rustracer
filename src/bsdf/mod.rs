use std::mem;
use na;

use ::Vector;
use colour::Colourf;

bitflags! {
    pub flags BxDFType: u32 {
        const REFLECTION   = 0b_0000_0001,
        const TRANSMISSION = 0b_0000_0010,
        const DIFFUSE      = 0b_0000_0100,
        const GLOSSY       = 0b_0000_1000,
        const SPECULAR     = 0b_0001_0000,
    }
}

/// Represents the Bidirectional Scattering Distribution Function.
/// It represents the properties of a material at a given point.
pub struct BSDF {
    /// Index of refraction of the surface
    eta: f32,
    /// Shading normal (i.e. potentially affected by bump-mapping)
    ns: Vector,
    /// Geometry normal
    ng: Vector, // bxdfs: BxDFType,
}

impl BSDF {
    pub fn new(n: Vector) -> Self {
        BSDF {
            eta: 1.0,
            ns: n,
            ng: n,
        }
    }

    /// Evaluate the BSDF for the given incoming light direction and outgoing light direction.
    pub fn f(&self, wi: &Vector, wo: &Vector) -> Colourf {
        Colourf::rgb(0.7, 0.7, 0.7)
    }

    pub fn sample_f(&self,
                    wo: &Vector,
                    sample: (f32, f32),
                    flags: BxDFType)
                    -> (Colourf, Vector, f32) {
        // TODO implement
        (Colourf::black(), na::zero(), 0.0)
        // for bxdf in self.bxdfs {
        //     if bxdf.matches(&flags) {
        //     }
        // }
    }
}

trait BxDF {
    fn matches(&self, flags: BxDFType) -> bool;
}
