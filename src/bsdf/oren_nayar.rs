use std::f32::consts;

use ::Vector;
use spectrum::Spectrum;
use bsdf::{BxDFType, BSDF_REFLECTION, BSDF_DIFFUSE};
use bsdf::{sin_theta, sin_phi, cos_phi, abs_cos_theta};
use bsdf::bxdf::BxDF;

pub struct OrenNayar {
    r: Spectrum,
    a: f32,
    b: f32,
}

impl OrenNayar {
    pub fn new(r: Spectrum, sigma: f32) -> OrenNayar {
        let sigma_rad = sigma.to_radians();
        let sigma2 = sigma * sigma;

        OrenNayar {
            r: r,
            a: 1.0 - (sigma2 / (2.0 * (sigma2 + 0.33))),
            b: 0.45 * sigma2 / (sigma2 + 0.09),
        }
    }
}

impl BxDF for OrenNayar {
    fn f(&self, wo: &Vector, wi: &Vector) -> Spectrum {
        let sin_theta_i = sin_theta(wi);
        let sin_theta_o = sin_theta(wo);

        // compute cosine term of the Oren-Nayar model
        let mut max_cos = 0.0;
        if sin_theta_i > 1e-4 && sin_theta_o > 1e-4 {
            let sin_phi_i = sin_phi(wi);
            let cos_phi_i = cos_phi(wi);
            let sin_phi_o = sin_phi(wo);
            let cos_phi_o = cos_phi(wo);
            let d_cos = sin_phi_i * sin_phi_o + cos_phi_i * cos_phi_o;
            max_cos = d_cos.max(0.0);
        }
        // compute sine and tangent terms of Oren-Nayar model
        let (sin_alpha, tan_beta) = if abs_cos_theta(wi) > abs_cos_theta(wo) {
            (sin_theta_o, sin_theta_i / abs_cos_theta(wi))
        } else {
            (sin_theta_i, sin_theta_o / abs_cos_theta(wo))
        };

        self.r * consts::FRAC_1_PI * (self.a + self.b * max_cos * sin_alpha * tan_beta)
    }

    fn get_type(&self) -> BxDFType {
        BSDF_REFLECTION | BSDF_DIFFUSE
    }
}
