use std::f32::consts;
use std::fmt::Debug;

use super::BxDFType;
use {Point2f, Vector3f};
use geometry::{abs_cos_theta, same_hemisphere};
use sampling::cosine_sample_hemisphere;
use spectrum::Spectrum;

pub trait BxDF: Debug {
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

#[derive(Debug, Clone, Copy)]
pub struct ScaledBxDF<'a> {
    bxdf: &'a BxDF,
    scale: Spectrum,
}

impl<'a> ScaledBxDF<'a> {
    pub fn new(bxdf: &'a BxDF, scale: Spectrum) -> ScaledBxDF<'a> {
        ScaledBxDF {
            bxdf: bxdf,
            scale: scale,
        }
    }
}

impl<'a> BxDF for ScaledBxDF<'a> {
    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum {
        self.bxdf.f(wo, wi) * self.scale
    }
    fn sample_f(&self, wo: &Vector3f, sample: &Point2f) -> (Spectrum, Vector3f, f32, BxDFType) {
        let (spectrum, wi, pdf, bxdftype) = self.bxdf.sample_f(wo, sample);
        (spectrum * self.scale, wi, pdf, bxdftype)
    }

    fn get_type(&self) -> BxDFType {
        self.bxdf.get_type()
    }
}
