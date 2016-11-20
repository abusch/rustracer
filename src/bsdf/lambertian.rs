use std::f32::consts;
use super::*;
use ::{Vector, Point2f};
use spectrum::Spectrum;

pub struct LambertianReflection {
    r: Spectrum,
}

impl LambertianReflection {
    pub fn new(r: Spectrum) -> LambertianReflection {
        LambertianReflection { r: r }
    }
}

impl BxDF for LambertianReflection {
    fn f(&self, wo: &Vector, wi: &Vector) -> Spectrum {
        self.r * consts::FRAC_1_PI
    }

    fn get_type(&self) -> BxDFType {
        BSDF_DIFFUSE | BSDF_REFLECTION
    }
}
