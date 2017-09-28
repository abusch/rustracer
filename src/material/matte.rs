use std::sync::Arc;
use std::path::Path;

use bsdf::{BxDF, LambertianReflection, OrenNayar, BSDF};
use interaction::SurfaceInteraction;
use material::{Material, TransportMode};
use paramset::TextureParams;
use spectrum::Spectrum;
use texture::{CheckerboardTexture, ConstantTexture, ImageTexture, Texture, UVTexture};

pub struct MatteMaterial {
    kd: Arc<Texture<Spectrum> + Sync + Send>,
    sigma: Arc<Texture<f32> + Sync + Send>,
}

impl MatteMaterial {
    pub fn new(r: Spectrum, sigma: f32) -> MatteMaterial {
        MatteMaterial {
            kd: Arc::new(ConstantTexture::new(r)),
            sigma: Arc::new(ConstantTexture::new(sigma)),
        }
    }

    pub fn new_uv_texture() -> MatteMaterial {
        MatteMaterial {
            kd: Arc::new(UVTexture::new()),
            sigma: Arc::new(ConstantTexture::new(0.0)),
        }
    }

    pub fn new_image(kd: &str) -> MatteMaterial {
        MatteMaterial {
            kd: Arc::new(ImageTexture::new(Path::new(kd))),
            sigma: Arc::new(ConstantTexture::new(0.5)),
        }
    }

    pub fn checkerboard(sigma: f32) -> MatteMaterial {
        MatteMaterial {
            kd: Arc::new(CheckerboardTexture::bw()),
            sigma: Arc::new(ConstantTexture::new(sigma)),
        }
    }

    pub fn bsdf(&self, si: &SurfaceInteraction) -> BSDF {
        let mut bxdfs: Vec<Box<BxDF + Send + Sync>> = Vec::new();
        let r = self.kd.evaluate(si);
        let sigma = self.sigma.evaluate(si);
        if sigma == 0.0 {
            bxdfs.push(Box::new(LambertianReflection::new(r)));
        } else {
            bxdfs.push(Box::new(OrenNayar::new(r, sigma)));
        }

        BSDF::new(si, 1.5, bxdfs)
    }

    pub fn create(mp: &mut TextureParams) -> Arc<Material + Send + Sync> {
        let kd = mp.get_spectrum_texture("Kd", &Spectrum::grey(0.5));
        let sigma = mp.get_float_texture("sigma", 0.0);

        Arc::new(MatteMaterial {
            kd: kd,
            sigma: sigma,
        })
    }
}

impl Material for MatteMaterial {
    fn compute_scattering_functions(
        &self,
        si: &mut SurfaceInteraction,
        _mode: TransportMode,
        _allow_multiple_lobes: bool,
    ) {
        si.bsdf = Some(Arc::new(self.bsdf(si)));
    }
}

impl Default for MatteMaterial {
    fn default() -> Self {
        MatteMaterial::new(Spectrum::grey(0.5), 0.0)
    }
}
