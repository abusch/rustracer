use std::sync::Arc;

use bsdf::{BxDF, Fresnel, SpecularReflection, BSDF};
use interaction::SurfaceInteraction;
use material::{Material, TransportMode};
use paramset::TextureParams;
use texture::Texture;
use spectrum::Spectrum;

pub struct MirrorMaterial {
    kr: Arc<Texture<Spectrum> + Send + Sync>,
}

impl MirrorMaterial {
    pub fn create(mp: &mut TextureParams) -> Arc<Material + Sync + Send> {
        let Kr = mp.get_spectrum_texture("Kr", &Spectrum::grey(0.9));
        // TODO bumpmap

        Arc::new(MirrorMaterial { kr: Kr })
    }
}

impl Material for MirrorMaterial {
    fn compute_scattering_functions(
        &self,
        si: &mut SurfaceInteraction,
        mode: TransportMode,
        allow_multiple_lobes: bool,
    ) {
        // TODO bumpmap
        let mut bxdfs: Vec<Box<BxDF + Send + Sync>> = Vec::new();
        let R = self.kr.evaluate(si); // TODO clamp
        if R.is_black() {
            bxdfs.push(Box::new(
                SpecularReflection::new(R, Box::new(Fresnel::no_op())),
            ));
        }
        si.bsdf = Some(Arc::new(BSDF::new(si, 1.0, bxdfs)));
    }
}
