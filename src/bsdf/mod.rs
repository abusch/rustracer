use std::mem;
use na::{Cross, Dot, zero, Norm, clamp};

use ::Vector;
use colour::Colourf;
use intersection::Intersection;
use interaction::SurfaceInteraction;

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
#[derive(Copy, Clone, Debug)]
pub struct BSDF {
    /// Index of refraction of the surface
    eta: f32,
    /// Shading normal (i.e. potentially affected by bump-mapping)
    ns: Vector,
    /// Geometry normal
    ng: Vector,
    ss: Vector,
    ts: Vector, // bxdfs: BxDFType,
}

impl BSDF {
    pub fn new(isect: &Intersection, eta: f32) -> Self {
        let n = isect.dg.nhit;
        let ss = isect.dg.dpdu.normalize();
        BSDF {
            eta: eta,
            ns: n,
            ng: n,
            ss: ss,
            ts: n.cross(&ss),
        }
    }

    pub fn new2(isect: &SurfaceInteraction, eta: f32) -> Self {
        let ss = isect.dpdu.normalize();
        BSDF {
            eta: eta,
            ns: isect.shading.n,
            ng: isect.n,
            ss: ss,
            ts: isect.shading.n.cross(&ss),
        }
    }

    /// Evaluate the BSDF for the given incoming light direction and outgoing light direction.
    pub fn f(&self, _wi_w: &Vector, _wo_w: &Vector) -> Colourf {
        Colourf::black()
    }

    pub fn sample_f(&self,
                    wo_w: &Vector,
                    sample: (f32, f32),
                    flags: BxDFType)
                    -> (Colourf, Vector, f32) {
        if !flags.contains(SPECULAR) {
            unimplemented!();
        }

        if flags.contains(REFLECTION) {
            let wo = self.world_to_local(&wo_w);
            let wi = Vector::new(-wo.x, -wo.y, wo.z);
            let cos_theta_i = wi.z;
            let kr = fresnel(cos_theta_i, 1.0, self.eta);
            let colour = Colourf::rgb(1.0, 1.0, 1.0) * kr / cos_theta_i.abs();

            assert!(!colour.has_nan());
            return (colour, self.local_to_world(&wi), 1.0);
        } else if flags.contains(TRANSMISSION) {
            let wo = self.world_to_local(&wo_w);
            let entering = wo.z > 0.0;
            let (eta_i, eta_t) = if entering {
                (1.0, self.eta)
            } else {
                (self.eta, 1.0)
            };
            let n = if wo.z < 0.0 {
                -Vector::z()
            } else {
                Vector::z()
            };
            return refract(&wo, &n, eta_i / eta_t)
                .map(|wi| {
                    let cos_theta_i = wi.z;
                    let kr = fresnel(cos_theta_i, 1.0, self.eta);
                    let colour = Colourf::rgb(1.0, 1.0, 1.0) * (1.0 - kr) / cos_theta_i.abs();

                    assert!(!colour.has_nan());
                    (colour, self.local_to_world(&wi), 1.0)
                })
                .unwrap_or((Colourf::black(), zero(), 0.0));
        }

        (Colourf::black(), zero(), 0.0)
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

trait BxDF {
    fn matches(&self, flags: BxDFType) -> bool;
}


/// Compute the reflection direction
fn reflect(wo: &Vector, n: &Vector) -> Vector {
    (-(*wo) + *n * 2.0 * wo.dot(n)).normalize()
}

/// Compute the refraction direction
fn refract(i: &Vector, n: &Vector, eta: f32) -> Option<Vector> {
    let cos_theta_i = n.dot(i);
    let sin2theta_i = (1.0 - cos_theta_i * cos_theta_i).max(0.0);
    let sin2theta_t = eta * eta * sin2theta_i;

    if sin2theta_t >= 1.0 {
        None
    } else {
        let cos_theta_t = (1.0 - sin2theta_t).sqrt();
        Some(eta * -*i + (eta * cos_theta_i - cos_theta_t) * *n)
    }
}

/// Compute the Fresnel coefficient
fn fresnel(cos_theta_i: f32, eta_i: f32, eta_t: f32) -> f32 {
    let mut cos_theta_i = clamp(cos_theta_i, -1.0, 1.0);
    let (mut eta_i, mut eta_t) = (eta_i, eta_t);
    if cos_theta_i <= 0.0 {
        // If leaving the surface, swap the indices of refraction
        mem::swap(&mut eta_i, &mut eta_t);
        cos_theta_i = cos_theta_i.abs();
    }

    let sin_theta_t = eta_i / eta_t * (1.0 - cos_theta_i * cos_theta_i).max(0.0).sqrt();
    if sin_theta_t >= 1.0 {
        1.0
    } else {
        let cos_theta_t = (1.0 - sin_theta_t * sin_theta_t).max(0.0).sqrt();
        let r_s = ((eta_t * cos_theta_i) - (eta_i * cos_theta_t)) /
                  ((eta_t * cos_theta_i) + (eta_i * cos_theta_t));
        let r_p = ((eta_i * cos_theta_i) - (eta_t * cos_theta_t)) /
                  ((eta_i * cos_theta_i) + (eta_t * cos_theta_t));
        (r_s * r_s + r_p * r_p) / 2.0
    }
}
