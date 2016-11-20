use std::sync::Arc;

use bsdf::{BSDF, BxDF, FresnelConductor, SpecularReflection, LambertianReflection, OrenNayar};
use colour::Colourf;
use material::{Material, TransportMode};
use interaction::SurfaceInteraction;
use texture::{Texture, ConstantTexture};

pub struct MatteMaterial {
    kd: Arc<Texture<Colourf> + Sync + Send>,
    sigma: f32,
}

impl MatteMaterial {
    pub fn new(r: Colourf, sigma: f32) -> MatteMaterial {
        MatteMaterial {
            kd: Arc::new(ConstantTexture::new(r)),
            sigma: sigma, /* bxdfs: vec![Box::new(SpecularReflection::new(Colourf::rgb(1.0, 0.0, 0.0),
                           *                                              Box::new(FresnelConductor::new(
                           *                                                      Colourf::white(),
                           *                                                      Colourf::rgb(0.155265, 0.116723, 0.138381),
                           *                                                      Colourf::rgb(4.82835, 3.12225, 2.14696),
                           *                                                      ))))], */
        }
    }

    pub fn bsdf(&self, si: &SurfaceInteraction) -> BSDF {
        let mut bxdfs: Vec<Box<BxDF + Send + Sync>> = Vec::new();
        let r = self.kd.evaluate(si);
        if self.sigma == 0.0 {
            bxdfs.push(Box::new(LambertianReflection::new(r)));
        } else {
            bxdfs.push(Box::new(OrenNayar::new(r, self.sigma)));
        }

        BSDF::new(si, 1.5, bxdfs)
    }
}

impl Material for MatteMaterial {
    fn compute_scattering_functions(&self,
                                    si: &mut SurfaceInteraction,
                                    mode: TransportMode,
                                    allow_multiple_lobes: bool) {
        si.bsdf = Some(Arc::new(self.bsdf(si)));
    }
}
