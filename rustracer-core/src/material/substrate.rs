use std::sync::Arc;

use light_arena::Allocator;

use crate::bsdf::{Bsdf, BxDFHolder, FresnelBlend, TrowbridgeReitzDistribution};
use crate::interaction::SurfaceInteraction;
use crate::material::{Material, TransportMode};
use crate::paramset::TextureParams;
use crate::spectrum::Spectrum;
use crate::texture::{TextureFloat, TextureSpectrum};

#[derive(Debug)]
pub struct SubstrateMaterial {
    kd: Arc<TextureSpectrum>,
    ks: Arc<TextureSpectrum>,
    nu: Arc<TextureFloat>,
    nv: Arc<TextureFloat>,
    bump_map: Option<Arc<TextureFloat>>,
    remap_roughness: bool,
}

impl SubstrateMaterial {
    pub fn create(mp: &TextureParams<'_>) -> Arc<dyn Material> {
        let kd = mp.get_spectrum_texture("Kd", &Spectrum::grey(0.5));
        let ks = mp.get_spectrum_texture("Ks", &Spectrum::grey(0.5));
        let urough = mp.get_float_texture("uroughness", 0.1);
        let vrough = mp.get_float_texture("vroughness", 0.1);
        let bump_map = mp.get_float_texture_or_none("bumpmap");
        let remap_roughness = mp.find_bool("remaproughness", true);

        Arc::new(SubstrateMaterial {
            kd,
            ks,
            nu: urough,
            nv: vrough,
            bump_map,
            remap_roughness,
        })
    }
}

impl Material for SubstrateMaterial {
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

        let d = self.kd.evaluate(si).clamp();
        let s = self.ks.evaluate(si).clamp();
        let mut roughu = self.nu.evaluate(si);
        let mut roughv = self.nv.evaluate(si);

        if !d.is_black() || !s.is_black() {
            if self.remap_roughness {
                roughu = TrowbridgeReitzDistribution::roughness_to_alpha(roughu);
                roughv = TrowbridgeReitzDistribution::roughness_to_alpha(roughv);
            }
            let distrib = arena.alloc(TrowbridgeReitzDistribution::new(roughu, roughv));
            bxdfs.add(arena.alloc(FresnelBlend::new(s, d, distrib)));
        }

        let bsdf = Bsdf::new(si, 1.0, bxdfs.into_slice());
        si.bsdf = Some(Arc::new(bsdf));
    }
}
