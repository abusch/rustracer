use std::mem;
use na::{Dot, zero, Norm, clamp};

use ::Vector;
use colour::Colourf;
use ray::Ray;

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
        Colourf::black()
    }

    pub fn sample_f(&self,
                    wo: &Vector,
                    sample: (f32, f32),
                    flags: BxDFType)
                    -> (Colourf, Vector, f32) {
        if !flags.contains(SPECULAR) {
            unimplemented!();
        }

        if flags.contains(REFLECTION) {
            let dir = -(*wo);
            let kr = fresnel(&dir, &self.ns, 1.5);
            let wi = reflect(&dir, &self.ns);

            return (Colourf::rgb(1.0, 1.0, 1.0), -wi, 1.0);
        }

        (Colourf::black(), zero(), 0.0)
    }
}

trait BxDF {
    fn matches(&self, flags: BxDFType) -> bool;
}


/// Compute the reflection direction
fn reflect(i: &Vector, n: &Vector) -> Vector {
    (*i - *n * 2.0 * n.dot(i)).normalize()
}

/// Compute the refraction direction
fn refract(i: &Vector, n: &Vector, ior: f32) -> Vector {
    let mut cos_i = clamp(i.dot(n), -1.0, 1.0);
    let (etai, etat, n_refr) = if cos_i < 0.0 {
        cos_i = -cos_i;
        (1.0, ior, *n)
    } else {
        (ior, 1.0, -*n)
    };

    let eta = etai / etat;
    let k = 1.0 - eta * eta * (1.0 - cos_i * cos_i);

    if k > 0.0 {
        *i * eta + n_refr * (eta * cos_i - k.sqrt())
    } else {
        zero()
    }
}

/// Compute the Fresnel coefficient
fn fresnel(i: &Vector, n: &Vector, ior: f32) -> f32 {
    let mut cosi = clamp(i.dot(n), -1.0, 1.0);
    let (etai, etat) = if cosi > 0.0 { (ior, 1.0) } else { (1.0, ior) };

    let sint = etai / etat * (1.0 - cosi * cosi).max(0.0).sqrt();
    if sint >= 1.0 {
        1.0
    } else {
        let cost = (1.0 - sint * sint).max(0.0).sqrt();
        cosi = cosi.abs();
        let r_s = ((etat * cosi) - (etai * cost)) / ((etat * cosi) + (etai * cost));
        let r_p = ((etai * cosi) - (etat * cost)) / ((etai * cosi) + (etat * cost));
        (r_s * r_s + r_p * r_p) / 2.0
    }
}
