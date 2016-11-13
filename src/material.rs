use std::sync::Arc;
use bsdf::{BSDF, BxDF, FresnelConductor, SpecularReflection, LambertianReflection};
use colour::Colourf;
use interaction::SurfaceInteraction;

pub enum TransportMode {
    RADIANCE,
    IMPORTANCE,
}

pub trait Material {
    fn compute_scattering_functions(&self,
                                    isect: &mut SurfaceInteraction,
                                    mode: TransportMode,
                                    allow_multiple_lobes: bool);
}

pub struct MatteMaterial {
    r: Colourf,
}

impl MatteMaterial {
    pub fn new(r: Colourf) -> MatteMaterial {
        MatteMaterial {
            r: r, /* bxdfs: vec![Box::new(SpecularReflection::new(Colourf::rgb(1.0, 0.0, 0.0),
                   *                                              Box::new(FresnelConductor::new(
                   *                                                      Colourf::white(),
                   *                                                      Colourf::rgb(0.155265, 0.116723, 0.138381),
                   *                                                      Colourf::rgb(4.82835, 3.12225, 2.14696),
                   *                                                      ))))], */
        }
    }

    pub fn bsdf(&self, isect: &SurfaceInteraction) -> BSDF {
        BSDF::new(isect,
                  1.5,
                  vec![Box::new(LambertianReflection::new(self.r))])
    }
}

impl Material for MatteMaterial {
    fn compute_scattering_functions(&self,
                                    isect: &mut SurfaceInteraction,
                                    mode: TransportMode,
                                    allow_multiple_lobes: bool) {
        isect.bsdf = Some(Arc::new(self.bsdf(isect)));
    }
}
