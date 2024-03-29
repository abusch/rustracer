use std::sync::Arc;

use light_arena::Allocator;
use log::info;

use crate::bsdf::{
    dielectric, Bsdf, BxDF, BxDFHolder, FresnelSpecular, MicrofacetReflection,
    MicrofacetTransmission, SpecularReflection, SpecularTransmission, TrowbridgeReitzDistribution,
};
use crate::interaction::SurfaceInteraction;
use crate::material::{Material, TransportMode};
use crate::paramset::TextureParams;
use crate::spectrum::Spectrum;
use crate::texture::{TextureFloat, TextureSpectrum};

#[derive(Debug)]
pub struct GlassMaterial {
    kr: Arc<TextureSpectrum>,
    kt: Arc<TextureSpectrum>,
    u_roughness: Arc<TextureFloat>,
    v_roughness: Arc<TextureFloat>,
    index: Arc<TextureFloat>,
    bump_map: Option<Arc<TextureFloat>>,
    remap_roughness: bool,
}

impl GlassMaterial {
    pub fn create(mp: &TextureParams<'_>) -> Arc<dyn Material> {
        info!("Creating Glass material");
        let Kr = mp.get_spectrum_texture("Kr", &Spectrum::white());
        let Kt = mp.get_spectrum_texture("Kt", &Spectrum::white());
        let eta = mp
            .get_float_texture_or_none("eta")
            .unwrap_or_else(|| mp.get_float_texture("index", 1.5));
        let rough_u = mp.get_float_texture("uroughness", 0.0);
        let rough_v = mp.get_float_texture("vroughness", 0.0);
        let bump_map = mp.get_float_texture_or_none("bumpmap");
        let remap_roughness = mp.find_bool("remaproughness", true);

        Arc::new(GlassMaterial {
            kr: Kr,
            kt: Kt,
            u_roughness: rough_u,
            v_roughness: rough_v,
            index: eta,
            bump_map,
            remap_roughness,
        })
    }
}

impl Material for GlassMaterial {
    fn compute_scattering_functions<'a, 'b>(
        &self,
        si: &mut SurfaceInteraction<'a, 'b>,
        mode: TransportMode,
        allow_multiple_lobes: bool,
        arena: &'b Allocator<'_>,
    ) {
        if let Some(ref bump) = self.bump_map {
            super::bump(bump, si);
        }
        let eta = self.index.evaluate(si);
        let mut u_rough = self.u_roughness.evaluate(si);
        let mut v_rough = self.v_roughness.evaluate(si);
        let r = self.kr.evaluate(si);
        let t = self.kt.evaluate(si);

        let mut bxdfs = BxDFHolder::new(arena);

        if !r.is_black() || !t.is_black() {
            let is_specular = u_rough == 0.0 && v_rough == 0.0;
            if is_specular && allow_multiple_lobes {
                bxdfs.add(arena.alloc(FresnelSpecular::new(r, t, 1.0, eta, mode)));
            } else {
                if self.remap_roughness {
                    u_rough = TrowbridgeReitzDistribution::roughness_to_alpha(u_rough);
                    v_rough = TrowbridgeReitzDistribution::roughness_to_alpha(v_rough);
                }
                if !r.is_black() {
                    let fresnel = arena.alloc(dielectric(1.0, eta));
                    let bxdf: &'b dyn BxDF = if is_specular {
                        arena.alloc(SpecularReflection::new(r, fresnel))
                    } else {
                        let distrib =
                            arena.alloc(TrowbridgeReitzDistribution::new(u_rough, v_rough));
                        arena.alloc(MicrofacetReflection::new(r, distrib, fresnel))
                    };
                    bxdfs.add(bxdf);
                }
                if !t.is_black() {
                    let bxdf: &'b dyn BxDF = if is_specular {
                        arena.alloc(SpecularTransmission::new(t, 1.0, eta, mode))
                    } else {
                        let distrib =
                            arena.alloc(TrowbridgeReitzDistribution::new(u_rough, v_rough));
                        arena.alloc(MicrofacetTransmission::new(r, distrib, 1.0, eta, mode))
                    };
                    bxdfs.add(bxdf);
                }
            }
        }

        let bsdf = Bsdf::new(si, eta, bxdfs.into_slice());
        si.bsdf = Some(Arc::new(bsdf));
    }
}
