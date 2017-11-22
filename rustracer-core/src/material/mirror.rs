use std::sync::Arc;

use light_arena::Allocator;

use bsdf::{BxDFHolder, Fresnel, SpecularReflection, BSDF};
use interaction::SurfaceInteraction;
use material::{Material, TransportMode};
use paramset::TextureParams;
use spectrum::Spectrum;
use texture::{TextureSpectrum, TextureFloat};

#[derive(Debug)]
pub struct MirrorMaterial {
    kr: Arc<TextureSpectrum>,
    bump_map: Option<Arc<TextureFloat>>,
}

impl MirrorMaterial {
    pub fn create(mp: &mut TextureParams) -> Arc<Material + Sync + Send> {
        info!("Creating Mirror material");
        let Kr = mp.get_spectrum_texture("Kr", &Spectrum::grey(0.9));
        let bump_map = mp.get_float_texture_or_none("bumpmap");

        Arc::new(MirrorMaterial { kr: Kr, bump_map })
    }
}

impl Material for MirrorMaterial {
    fn compute_scattering_functions<'a, 'b>(&self,
                                            si: &mut SurfaceInteraction<'a, 'b>,
                                            _mode: TransportMode,
                                            _allow_multiple_lobes: bool,
                                            arena: &'b Allocator) {
        if let Some(ref bump) = self.bump_map {
            super::bump(bump, si);
        }
        let mut bxdfs = BxDFHolder::new(arena);
        let R = self.kr.evaluate(si); // TODO clamp
        if !R.is_black() {
            let fresnel = arena <- Fresnel::no_op();
            bxdfs.add(arena <- SpecularReflection::new(R, fresnel));
        }

        si.bsdf = Some(Arc::new(BSDF::new(si, 1.0, bxdfs.into_slice())));
    }
}
