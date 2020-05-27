use std::sync::Arc;

use light_arena::Allocator;
use log::info;

use crate::bsdf::{BxDFHolder, LambertianReflection, OrenNayar, BSDF};
use crate::clamp;
use crate::interaction::SurfaceInteraction;
use crate::material::{Material, TransportMode};
use crate::paramset::TextureParams;
use crate::spectrum::Spectrum;
use crate::texture::{TextureFloat, TextureSpectrum};

#[derive(Debug)]
pub struct MatteMaterial {
    kd: Arc<TextureSpectrum>,
    sigma: Arc<TextureFloat>,
    bump_map: Option<Arc<TextureFloat>>,
}

impl MatteMaterial {
    pub fn create(mp: &TextureParams<'_>) -> Arc<dyn Material> {
        info!("Creating Matte material");
        let kd = mp.get_spectrum_texture("Kd", &Spectrum::grey(0.5));
        let sigma = mp.get_float_texture("sigma", 0.0);
        let bump_map = mp.get_float_texture_or_none("bumpmap");

        Arc::new(MatteMaterial {
            kd,
            sigma,
            bump_map,
        })
    }
}

impl Material for MatteMaterial {
    fn compute_scattering_functions<'a, 'b>(
        &self,
        si: &mut SurfaceInteraction<'a, 'b>,
        _mode: TransportMode,
        _allow_multiple_lobes: bool,
        arena: &'b Allocator<'_>,
    ) {
        let mut bxdfs = BxDFHolder::new(arena);

        if let Some(ref bump_map) = self.bump_map {
            super::bump(bump_map, si);
        }

        let r = self.kd.evaluate(si).clamp();
        let sigma = clamp(self.sigma.evaluate(si), 0.0, 1.0);
        if !r.is_black() {
            if sigma == 0.0 {
                bxdfs.add(arena.alloc(LambertianReflection::new(r)));
            } else {
                bxdfs.add(arena.alloc(OrenNayar::new(r, sigma)));
            }
        }

        let bsdf = BSDF::new(si, 1.0, bxdfs.into_slice());
        si.bsdf = Some(Arc::new(bsdf));
    }
}
