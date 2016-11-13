use super::BxDFType;
use ::{Vector, Point2f};
use colour::Colourf;

pub trait BxDF {
    fn f(&self, wo: &Vector, wi: &Vector) -> Colourf;
    fn sample_f(&self, wo: &Vector, sample: &Point2f) -> (Vector, f32, Option<BxDFType>, Colourf);
    // fn rho(&self, wo: &Vector, n_samples: u32) -> (Point2f, Colourf);
    // fn rho_hh(&self, n_samples: u32) -> (Point2f, Point2f, Colourf);
    fn matches(&self, flags: BxDFType) -> bool {
        self.get_type() & flags == self.get_type()
    }

    fn get_type(&self) -> BxDFType;
}

pub struct ScaledBxDF {
    bxdf: Box<BxDF>,
    scale: Colourf,
}

impl ScaledBxDF {
    fn new(bxdf: Box<BxDF>, scale: Colourf) -> ScaledBxDF {
        ScaledBxDF {
            bxdf: bxdf,
            scale: scale,
        }
    }
}

impl BxDF for ScaledBxDF {
    fn f(&self, wo: &Vector, wi: &Vector) -> Colourf {
        self.bxdf.f(wo, wi) * self.scale
    }
    fn sample_f(&self, wo: &Vector, sample: &Point2f) -> (Vector, f32, Option<BxDFType>, Colourf) {
        let (wi, pdf, bxdftype, spectrum) = self.bxdf.sample_f(wo, sample);
        (wi, pdf, bxdftype, spectrum * self.scale)
    }
    // fn rho(&self, wo: &Vector, n_samples: u32) -> (Point2f, Colourf) {
    //     let (sample, spectrum) = self.bxdf.rho(wo, n_samples);
    //     (sample, spectrum * self.scale)
    // }
    // fn rho_hh(&self, n_samples: u32) -> (Point2f, Point2f, Colourf) {
    //     let (sample1, sample2, spectrum) = self.bxdf.rho_hh(n_samples);
    //     (sample1, sample2, spectrum * self.scale)
    // }
    fn get_type(&self) -> BxDFType {
        self.bxdf.get_type()
    }
}
