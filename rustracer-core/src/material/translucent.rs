use std::sync::Arc;

use light_arena::Allocator;

use crate::bsdf::{
    dielectric, Bsdf, BxDFHolder, LambertianReflection, LambertianTransmission,
    MicrofacetReflection, MicrofacetTransmission, TrowbridgeReitzDistribution,
};
use crate::interaction::SurfaceInteraction;
use crate::material::{Material, TransportMode};
use crate::paramset::TextureParams;
use crate::spectrum::Spectrum;
use crate::texture::{TextureFloat, TextureSpectrum};

#[derive(Debug)]
pub struct TranslucentMaterial {
    kd: Arc<TextureSpectrum>,
    ks: Arc<TextureSpectrum>,
    roughness: Arc<TextureFloat>,
    reflect: Arc<TextureSpectrum>,
    transmit: Arc<TextureSpectrum>,
    bumpmap: Option<Arc<TextureFloat>>,
    remap_roughness: bool,
}

impl TranslucentMaterial {
    pub fn create(mp: &TextureParams<'_>) -> Arc<dyn Material> {
        let kd = mp.get_spectrum_texture("Kd", &Spectrum::from(0.25));
        let ks = mp.get_spectrum_texture("Ks", &Spectrum::from(0.25));
        let reflect = mp.get_spectrum_texture("reflect", &Spectrum::from(0.5));
        let transmit = mp.get_spectrum_texture("transmit", &Spectrum::from(0.5));
        let roughness = mp.get_float_texture("roughness", 0.1);
        let bumpmap = mp.get_float_texture_or_none("bumpmap");
        let remap_roughness = mp.find_bool("remaproughness", true);

        Arc::new(TranslucentMaterial {
            kd,
            ks,
            roughness,
            reflect,
            transmit,
            bumpmap,
            remap_roughness,
        })
    }
}

impl Material for TranslucentMaterial {
    fn compute_scattering_functions<'a, 'b>(
        &self,
        si: &mut SurfaceInteraction<'a, 'b>,
        mode: TransportMode,
        _allow_multiple_lobes: bool,
        arena: &'b Allocator<'_>,
    ) {
        let mut bxdfs = BxDFHolder::new(arena);
        let eta = 1.5;

        if let Some(ref bump_map) = self.bumpmap {
            super::bump(bump_map, si);
        }

        let r = self.reflect.evaluate(si).clamp();
        let t = self.transmit.evaluate(si).clamp();

        if !r.is_black() || !t.is_black() {
            let kd = self.kd.evaluate(si).clamp();
            if !kd.is_black() {
                if !r.is_black() {
                    bxdfs.add(arena.alloc(LambertianReflection::new(r * kd)));
                }
                if !t.is_black() {
                    bxdfs.add(arena.alloc(LambertianTransmission::new(t * kd)));
                }
            }
            let ks = self.ks.evaluate(si).clamp();
            if !ks.is_black() && (!r.is_black() || !t.is_black()) {
                let mut rough = self.roughness.evaluate(si);
                if self.remap_roughness {
                    rough = TrowbridgeReitzDistribution::roughness_to_alpha(rough);
                }
                let distrib = arena.alloc(TrowbridgeReitzDistribution::new(rough, rough));
                if !r.is_black() {
                    let fresnel = arena.alloc(dielectric(1.0, eta));
                    bxdfs.add(arena.alloc(MicrofacetReflection::new(r * ks, distrib, fresnel)));
                }
                if !t.is_black() {
                    bxdfs.add(arena.alloc(MicrofacetTransmission::new(
                        t * ks,
                        distrib,
                        1.0,
                        eta,
                        mode,
                    )));
                }
            }
        }

        let bsdf: Bsdf<'b> = Bsdf::new(si, eta, bxdfs.into_slice());
        si.bsdf = Some(Arc::new(bsdf));
    }
}
