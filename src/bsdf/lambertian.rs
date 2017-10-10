use std::f32::consts;

use bsdf::{BxDF, BxDFType};
use Vector3f;
use spectrum::Spectrum;

#[derive(Debug)]
pub struct LambertianReflection {
    r: Spectrum,
}

impl LambertianReflection {
    pub fn new(r: Spectrum) -> LambertianReflection {
        LambertianReflection { r: r }
    }
}

impl BxDF for LambertianReflection {
    fn f(&self, _wo: &Vector3f, _wi: &Vector3f) -> Spectrum {
        self.r * consts::FRAC_1_PI
    }

    fn get_type(&self) -> BxDFType {
        BxDFType::BSDF_DIFFUSE | BxDFType::BSDF_REFLECTION
    }
}
