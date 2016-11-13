use std::f32::consts;
use super::*;
use ::{Vector, Point2f};
use colour::Colourf;

pub struct LambertianReflection {
    r: Colourf,
}

impl LambertianReflection {
    pub fn new(r: Colourf) -> LambertianReflection {
        LambertianReflection { r: r }
    }
}

impl BxDF for LambertianReflection {
    fn f(&self, wo: &Vector, wi: &Vector) -> Colourf {
        self.r * consts::FRAC_1_PI
    }

    fn get_type(&self) -> BxDFType {
        BSDF_DIFFUSE | BSDF_REFLECTION
    }
}
