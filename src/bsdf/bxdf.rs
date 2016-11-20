use std::f32::consts;

use super::{BxDFType, abs_cos_theta, same_hemisphere};
use ::{Vector, Point2f};
use sampling::cosine_sample_hemisphere;
use spectrum::Spectrum;

pub trait BxDF {
    fn f(&self, wo: &Vector, wi: &Vector) -> Spectrum;
    fn sample_f(&self, wo: &Vector, u: &Point2f) -> (Vector, f32, Option<BxDFType>, Spectrum) {
        let mut wi = cosine_sample_hemisphere(u);
        if wo.z < 0.0 {
            wi.z *= -1.0;
        }
        let pdf = self.pdf(wo, &wi);
        (wi, pdf, None, self.f(wo, &wi))
    }
    // TODO
    // fn rho(&self, wo: &Vector, n_samples: u32) -> (Point2f, Spectrum);
    // fn rho_hh(&self, n_samples: u32) -> (Point2f, Point2f, Spectrum);
    fn matches(&self, flags: BxDFType) -> bool {
        self.get_type() & flags == self.get_type()
    }

    fn get_type(&self) -> BxDFType;

    fn pdf(&self, wo: &Vector, wi: &Vector) -> f32 {
        if same_hemisphere(wo, wi) {
            abs_cos_theta(wi) * consts::FRAC_1_PI
        } else {
            0.0
        }
    }
}

pub struct ScaledBxDF {
    bxdf: Box<BxDF>,
    scale: Spectrum,
}

impl ScaledBxDF {
    fn new(bxdf: Box<BxDF>, scale: Spectrum) -> ScaledBxDF {
        ScaledBxDF {
            bxdf: bxdf,
            scale: scale,
        }
    }
}

impl BxDF for ScaledBxDF {
    fn f(&self, wo: &Vector, wi: &Vector) -> Spectrum {
        self.bxdf.f(wo, wi) * self.scale
    }
    fn sample_f(&self, wo: &Vector, sample: &Point2f) -> (Vector, f32, Option<BxDFType>, Spectrum) {
        let (wi, pdf, bxdftype, spectrum) = self.bxdf.sample_f(wo, sample);
        (wi, pdf, bxdftype, spectrum * self.scale)
    }
    // fn rho(&self, wo: &Vector, n_samples: u32) -> (Point2f, Spectrum) {
    //     let (sample, spectrum) = self.bxdf.rho(wo, n_samples);
    //     (sample, spectrum * self.scale)
    // }
    // fn rho_hh(&self, n_samples: u32) -> (Point2f, Point2f, Spectrum) {
    //     let (sample1, sample2, spectrum) = self.bxdf.rho_hh(n_samples);
    //     (sample1, sample2, spectrum * self.scale)
    // }
    fn get_type(&self) -> BxDFType {
        self.bxdf.get_type()
    }
}
