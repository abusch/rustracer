use std::sync::Arc;

use light_arena::Allocator;
use log::info;

use crate::bsdf::{BxDFHolder, Fresnel, SpecularReflection, BSDF};
use crate::interaction::SurfaceInteraction;
use crate::material::{Material, TransportMode};
use crate::paramset::TextureParams;
use crate::spectrum::Spectrum;
use crate::texture::{TextureFloat, TextureSpectrum};

#[derive(Debug)]
pub struct MirrorMaterial {
    kr: Arc<TextureSpectrum>,
    bump_map: Option<Arc<TextureFloat>>,
}

impl MirrorMaterial {
    pub fn create(mp: &TextureParams<'_>) -> Arc<dyn Material> {
        info!("Creating Mirror material");
        let Kr = mp.get_spectrum_texture("Kr", &Spectrum::grey(0.9));
        let bump_map = mp.get_float_texture_or_none("bumpmap");

        Arc::new(MirrorMaterial { kr: Kr, bump_map })
    }
}

impl Material for MirrorMaterial {
    fn compute_scattering_functions<'a, 'b>(
        &self,
        si: &mut SurfaceInteraction<'a, 'b>,
        _mode: TransportMode,
        _allow_multiple_lobes: bool,
        arena: &'b Allocator<'_>,
    ) {
        if let Some(ref bump) = self.bump_map {
            super::bump(bump, si);
        }
        let mut bxdfs = BxDFHolder::new(arena);
        let R = self.kr.evaluate(si).clamp();
        if !R.is_black() {
            let fresnel = arena.alloc(Fresnel::no_op());
            bxdfs.add(arena.alloc(SpecularReflection::new(R, fresnel)));
        }

        si.bsdf = Some(Arc::new(BSDF::new(si, 1.0, bxdfs.into_slice())));
    }
}
