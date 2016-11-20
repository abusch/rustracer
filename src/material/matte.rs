use std::sync::Arc;

use bsdf::{BSDF, BxDF, FresnelConductor, SpecularReflection, LambertianReflection, OrenNayar};
use spectrum::Spectrum;
use material::{Material, TransportMode};
use interaction::SurfaceInteraction;
use texture::{Texture, ConstantTexture, CheckerboardTexture};

pub struct MatteMaterial {
    kd: Arc<Texture<Spectrum> + Sync + Send>,
    sigma: f32,
}

impl MatteMaterial {
    pub fn new(r: Spectrum, sigma: f32) -> MatteMaterial {
        MatteMaterial {
            kd: Arc::new(ConstantTexture::new(r)),
            sigma: sigma, /* bxdfs: vec![Box::new(SpecularReflection::new(Spectrum::rgb(1.0, 0.0, 0.0),
                           *                                              Box::new(FresnelConductor::new(
                           *                                                      Spectrum::white(),
                           *                                                      Spectrum::rgb(0.155265, 0.116723, 0.138381),
                           *                                                      Spectrum::rgb(4.82835, 3.12225, 2.14696),
                           *                                                      ))))], */
        }
    }

    pub fn checkerboard(sigma: f32) -> MatteMaterial {
        MatteMaterial {
            kd: Arc::new(CheckerboardTexture::bw()),
            sigma: sigma,
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
