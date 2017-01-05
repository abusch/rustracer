use std::sync::Arc;
use std::path::Path;

use bsdf::{BSDF, BxDF, LambertianReflection, OrenNayar};
use spectrum::Spectrum;
use material::{Material, TransportMode};
use interaction::SurfaceInteraction;
use texture::{Texture, ConstantTexture, CheckerboardTexture, ImageTexture, UVTexture};

pub struct MatteMaterial {
    kd: Arc<Texture<Spectrum> + Sync + Send>,
    sigma: f32,
}

impl MatteMaterial {
    pub fn new(r: Spectrum, sigma: f32) -> MatteMaterial {
        MatteMaterial {
            kd: Arc::new(ConstantTexture::new(r)),
            sigma: sigma,
        }
    }

    pub fn new_uv_texture() -> MatteMaterial {
        MatteMaterial {
            kd: Arc::new(UVTexture::new()),
            sigma: 0.0,
        }
    }

    pub fn new_image(kd: &str) -> MatteMaterial {
        MatteMaterial {
            kd: Arc::new(ImageTexture::new(Path::new(kd))),
            sigma: 0.0,
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
                                    _mode: TransportMode,
                                    _allow_multiple_lobes: bool) {
        si.bsdf = Some(Arc::new(self.bsdf(si)));
    }
}

impl Default for MatteMaterial {
    fn default() -> Self {
        MatteMaterial::new(Spectrum::grey(0.5), 0.0)
    }
}
