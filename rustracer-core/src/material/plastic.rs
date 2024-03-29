use std::sync::Arc;

use light_arena::Allocator;
use log::info;

use crate::bsdf::{
    dielectric, Bsdf, BxDFHolder, LambertianReflection, MicrofacetReflection,
    TrowbridgeReitzDistribution,
};
use crate::interaction::SurfaceInteraction;
use crate::material::{Material, TransportMode};
use crate::paramset::TextureParams;
use crate::spectrum::Spectrum;
use crate::texture::{TextureFloat, TextureSpectrum};

#[derive(Debug)]
pub struct Plastic {
    kd: Arc<TextureSpectrum>,
    ks: Arc<TextureSpectrum>,
    roughness: Arc<TextureFloat>,
    bump_map: Option<Arc<TextureFloat>>,
    remap_roughness: bool,
}

impl Plastic {
    pub fn create(mp: &TextureParams<'_>) -> Arc<dyn Material> {
        info!("Creating Plastic material");
        let Kd = mp.get_spectrum_texture("Kd", &Spectrum::grey(0.25));
        let Ks = mp.get_spectrum_texture("Ks", &Spectrum::grey(0.25));
        let roughness = mp.get_float_texture("roughness", 0.1);
        let bump_map = mp.get_float_texture_or_none("bumpmap");
        let remap_roughness = mp.find_bool("remaproughness", true);

        Arc::new(Plastic {
            kd: Kd,
            ks: Ks,
            roughness,
            bump_map,
            remap_roughness,
        })
    }
}

impl Material for Plastic {
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
        let kd = self.kd.evaluate(si);
        let ks = self.ks.evaluate(si);

        let mut bxdfs = BxDFHolder::new(arena);
        if !kd.is_black() {
            bxdfs.add(arena.alloc(LambertianReflection::new(kd)));
        }
        if !ks.is_black() {
            let fresnel = arena.alloc(dielectric(1.5, 1.0));
            let mut roughness = self.roughness.evaluate(si);
            if self.remap_roughness {
                roughness = TrowbridgeReitzDistribution::roughness_to_alpha(roughness);
            }
            let distrib = arena.alloc(TrowbridgeReitzDistribution::new(roughness, roughness));
            bxdfs.add(arena.alloc(MicrofacetReflection::new(ks, distrib, fresnel)));
        }

        let bsdf: Bsdf<'b> = Bsdf::new(si, 1.0, bxdfs.into_slice());
        si.bsdf = Some(Arc::new(bsdf));
    }
}
