mod bxdf;
mod fresnel;
mod lambertian;
mod oren_nayar;
mod microfacet;

pub use self::bxdf::*;
pub use self::fresnel::*;
pub use self::lambertian::*;
pub use self::oren_nayar::*;
pub use self::microfacet::*;

use na::{self, Cross, Dot, zero, Norm};

use ::Vector;
use spectrum::Spectrum;
use interaction::SurfaceInteraction;

bitflags! {
    pub flags BxDFType: u32 {
        const BSDF_REFLECTION   = 0b_00000001,
        const BSDF_TRANSMISSION = 0b_00000010,
        const BSDF_DIFFUSE      = 0b_00000100,
        const BSDF_GLOSSY       = 0b_00001000,
        const BSDF_SPECULAR     = 0b_00010000,
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
    ng: Vector,
    ss: Vector,
    ts: Vector,
    bxdfs: Vec<Box<BxDF + Sync + Send>>,
}

impl BSDF {
    pub fn new(isect: &SurfaceInteraction, eta: f32, bxdfs: Vec<Box<BxDF + Sync + Send>>) -> BSDF {
        let ss = isect.dpdu.normalize();
        BSDF {
            eta: eta,
            ns: isect.shading.n,
            ng: isect.n,
            ss: ss,
            ts: isect.shading.n.cross(&ss),
            bxdfs: bxdfs,
        }
    }

    /// Evaluate the BSDF for the given incoming light direction and outgoing light direction.
    pub fn f(&self, wo_w: &Vector, wi_w: &Vector, flags: BxDFType) -> Spectrum {
        let wi = self.world_to_local(wi_w);
        let wo = self.world_to_local(wo_w);
        if wo.z == 0.0 {
            return Spectrum::black();
        }
        let reflect = wi_w.dot(&self.ng) * wo_w.dot(&self.ng) > 0.0;
        self.bxdfs
            .iter()
            .filter(|b| {
                // Make sure we only evaluate reflection or transmission based on whether wi and wo
                // lie in the same hemisphere.
                b.matches(flags) &&
                ((reflect && (b.get_type().contains(BSDF_REFLECTION))) ||
                 (!reflect && (b.get_type().contains(BSDF_TRANSMISSION))))
            })
            .fold(Spectrum::black(), |c, b| c + b.f(&wo, &wi))
    }

    pub fn sample_f(&self,
                    _wo_w: &Vector,
                    _sample: (f32, f32),
                    _flags: BxDFType)
                    -> (Spectrum, Vector, f32) {
        // if !flags.contains(BSDF_SPECULAR) {
        //     unimplemented!();
        // }

        // if flags.contains(BSDF_REFLECTION) {
        //     let wo = self.world_to_local(&wo_w);
        //     let wi = Vector::new(-wo.x, -wo.y, wo.z);
        //     let cos_theta_i = wi.z;
        //     let kr = fr_dielectric(cos_theta_i, 1.0, self.eta);
        //     let colour = Spectrum::rgb(1.0, 1.0, 1.0) * kr / cos_theta_i.abs();

        //     assert!(!colour.has_nan());
        //     return (colour, self.local_to_world(&wi), 1.0);
        // } else if flags.contains(BSDF_TRANSMISSION) {
        //     let wo = self.world_to_local(&wo_w);
        //     let entering = wo.z > 0.0;
        //     let (eta_i, eta_t) = if entering {
        //         (1.0, self.eta)
        //     } else {
        //         (self.eta, 1.0)
        //     };
        //     let n = if wo.z < 0.0 {
        //         -Vector::z()
        //     } else {
        //         Vector::z()
        //     };
        //     return refract(&wo, &n, eta_i / eta_t)
        //         .map(|wi| {
        //             let cos_theta_i = wi.z;
        //             let kr = fr_dielectric(cos_theta_i, 1.0, self.eta);
        //             let colour = Spectrum::rgb(1.0, 1.0, 1.0) * (1.0 - kr) / cos_theta_i.abs();

        //             assert!(!colour.has_nan());
        //             (colour, self.local_to_world(&wi), 1.0)
        //         })
        //         .unwrap_or((Spectrum::black(), zero(), 0.0));
        // }

        (Spectrum::black(), zero(), 0.0)
    }

    fn world_to_local(&self, v: &Vector) -> Vector {
        Vector::new(v.dot(&self.ss), v.dot(&self.ts), v.dot(&self.ns))
    }

    fn local_to_world(&self, v: &Vector) -> Vector {
        Vector::new(self.ss.x * v.x + self.ts.x * v.y + self.ns.x * v.z,
                    self.ss.y * v.x + self.ts.y * v.y + self.ns.y * v.z,
                    self.ss.z * v.z + self.ts.z * v.y + self.ns.z * v.z)
    }
}
