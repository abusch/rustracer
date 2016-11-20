use std::f32::consts;

use na::{Norm, Dot};

use ::Vector;
use colour::Colourf;
use bsdf::{BxDF, BxDFType, BSDF_REFLECTION, BSDF_GLOSSY};
use bsdf::{tan_theta, tan2_theta, abs_cos_theta, cos2_theta, cos_phi, cos2_phi, sin_phi, sin2_phi};
use bsdf::fresnel::Fresnel;

pub struct MicrofacetReflection {
    r: Colourf,
    distribution: Box<MicrofacetDistribution>,
    fresnel: Box<Fresnel>,
}

impl MicrofacetReflection {
    pub fn new(r: Colourf,
               distribution: Box<MicrofacetDistribution>,
               fresnel: Box<Fresnel>)
               -> MicrofacetReflection {
        MicrofacetReflection {
            r: r,
            distribution: distribution,
            fresnel: fresnel,
        }
    }
}

impl BxDF for MicrofacetReflection {
    fn f(&self, wi: &Vector, wo: &Vector) -> Colourf {
        let cos_theta_o = abs_cos_theta(wo);
        let cos_theta_i = abs_cos_theta(wi);
        let mut wh = *wi + *wo;

        // Handle degenerate case for microfacet reflection
        if cos_theta_o == 0.0 || cos_theta_i == 0.0 {
            return Colourf::black();
        }
        if wh.x == 0.0 && wh.y == 0.0 && wh.z == 0.0 {
            return Colourf::black();
        }

        wh = wh.normalize();
        let f = self.fresnel.evaluate(wi.dot(&wh));
        self.r * self.distribution.d(&wh) * self.distribution.g(wo, wi) * f /
        (4.0 * cos_theta_i * cos_theta_o)
    }

    fn get_type(&self) -> BxDFType {
        BSDF_REFLECTION | BSDF_GLOSSY
    }
}

// TODO MicrofacetTransmission

// Microfacet distributions
pub trait MicrofacetDistribution {
    fn d(&self, wh: &Vector) -> f32;
    fn lambda(&self, wh: &Vector) -> f32;
    fn g1(&self, wh: &Vector) -> f32 {
        1.0 / (1.0 + self.lambda(wh))
    }
    fn g(&self, wi: &Vector, wo: &Vector) -> f32 {
        1.0 / (1.0 + self.lambda(wi) + self.lambda(wo))
    }
}

pub struct BeckmannDistribution {
    alpha_x: f32,
    alpha_y: f32,
}

impl MicrofacetDistribution for BeckmannDistribution {
    fn d(&self, wh: &Vector) -> f32 {
        let tan2theta = tan2_theta(wh);
        if tan2theta.is_infinite() {
            return 0.0;
        }

        let cos4theta = cos2_theta(wh) * cos2_theta(wh);
        (-tan2theta *
         (cos2_phi(wh) / (self.alpha_x * self.alpha_x) +
          sin2_phi(wh) / (self.alpha_y * self.alpha_y)))
            .exp() / (consts::PI * self.alpha_x * self.alpha_y)
    }

    fn lambda(&self, wh: &Vector) -> f32 {
        let abs_tan_theta = tan_theta(wh).abs();
        if abs_tan_theta.is_infinite() {
            return 0.0;
        }

        // Compute alpha for direction w
        let alpha = (cos_phi(wh) * self.alpha_x * self.alpha_x +
                     sin_phi(wh) * self.alpha_y * self.alpha_y)
            .sqrt();

        let a = 1.0 / (alpha * abs_tan_theta);
        if a >= 1.6 {
            0.0
        } else {
            (1.0 - 1.259 * a + 0.396 * a * a) / (3.535 * a + 2.181 * a * a)
        }
    }
}

// TODO Trowbridge-Reitz distrbution
