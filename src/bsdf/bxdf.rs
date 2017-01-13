use std::f32::consts;

use super::BxDFType;
use ::{Vector3f, Point2f};
use geometry::{abs_cos_theta, same_hemisphere};
use sampling::cosine_sample_hemisphere;
use spectrum::Spectrum;

pub trait BxDF {
    /// Evaluate the BxDF for the given incoming and outgoing directions.
    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum;

    /// Sample the BxDF for the given outgoing direction, using the given pair of uniform samples.
    ///
    /// The default implementation uses importance sampling by using a cosine-weighted
    /// distribution.
    fn sample_f(&self, wo: &Vector3f, u: &Point2f) -> (Spectrum, Vector3f, f32, BxDFType) {
        let mut wi = cosine_sample_hemisphere(u);
        if wo.z < 0.0 {
            wi.z *= -1.0;
        }
        let pdf = self.pdf(wo, &wi);
        (self.f(wo, &wi), wi, pdf, BxDFType::empty())
    }
    // TODO implement rho functions
    // fn rho(&self, wo: &Vector3f, n_samples: u32) -> (Point2f, Spectrum);
    // fn rho_hh(&self, n_samples: u32) -> (Point2f, Point2f, Spectrum);
    fn matches(&self, flags: BxDFType) -> bool {
        self.get_type() & flags == self.get_type()
    }

    fn get_type(&self) -> BxDFType;

    /// Evaluate the PDF for the given outgoing and incoming directions.
    ///
    /// Note: this method needs to be consistent with ```BxDF::sample_f()```.
    fn pdf(&self, wo: &Vector3f, wi: &Vector3f) -> f32 {
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
    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum {
        self.bxdf.f(wo, wi) * self.scale
    }
    fn sample_f(&self, wo: &Vector3f, sample: &Point2f) -> (Spectrum, Vector3f, f32, BxDFType) {
        let (spectrum, wi, pdf, bxdftype) = self.bxdf.sample_f(wo, sample);
        (spectrum * self.scale, wi, pdf, bxdftype)
    }
    // fn rho(&self, wo: &Vector3f, n_samples: u32) -> (Point2f, Spectrum) {
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
