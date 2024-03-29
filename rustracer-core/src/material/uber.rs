use std::sync::Arc;

use light_arena::Allocator;

use crate::bsdf::{
    dielectric, Bsdf, BxDFHolder, LambertianReflection, MicrofacetReflection, SpecularReflection,
    SpecularTransmission, TrowbridgeReitzDistribution,
};
use crate::interaction::SurfaceInteraction;
use crate::material::{Material, TransportMode};
use crate::paramset::TextureParams;
use crate::spectrum::Spectrum;
use crate::texture::{TextureFloat, TextureSpectrum};

#[derive(Debug)]
pub struct UberMaterial {
    kd: Arc<TextureSpectrum>,
    ks: Arc<TextureSpectrum>,
    kr: Arc<TextureSpectrum>,
    kt: Arc<TextureSpectrum>,
    opacity: Arc<TextureSpectrum>,
    roughness: Arc<TextureFloat>,
    roughnessu: Option<Arc<TextureFloat>>,
    roughnessv: Option<Arc<TextureFloat>>,
    eta: Arc<TextureFloat>,
    bumpmap: Option<Arc<TextureFloat>>,
    remap_roughness: bool,
}

impl UberMaterial {
    pub fn create(mp: &TextureParams<'_>) -> Arc<dyn Material> {
        let kd = mp.get_spectrum_texture("Kd", &Spectrum::from(0.25));
        let ks = mp.get_spectrum_texture("Ks", &Spectrum::from(0.25));
        let kr = mp.get_spectrum_texture("Kr", &Spectrum::from(0.0));
        let kt = mp.get_spectrum_texture("Kt", &Spectrum::from(0.0));
        let roughness = mp.get_float_texture("roughness", 0.1);
        let uroughness = mp.get_float_texture_or_none("uroughness");
        let vroughness = mp.get_float_texture_or_none("vroughness");
        let eta = mp
            .get_float_texture_or_none("eta")
            .unwrap_or_else(|| mp.get_float_texture("index", 1.5));
        let opacity = mp.get_spectrum_texture("opacity", &Spectrum::from(1.0));
        let bumpmap = mp.get_float_texture_or_none("bumpmap");
        let remap_roughness = mp.find_bool("remaproughness", true);

        Arc::new(UberMaterial {
            kd,
            ks,
            kr,
            kt,
            opacity,
            roughness,
            roughnessu: uroughness,
            roughnessv: vroughness,
            eta,
            bumpmap,
            remap_roughness,
        })
    }
}

impl Material for UberMaterial {
    fn compute_scattering_functions<'a, 'b>(
        &self,
        si: &mut SurfaceInteraction<'a, 'b>,
        mode: TransportMode,
        _allow_multiple_lobes: bool,
        arena: &'b Allocator<'_>,
    ) {
        let mut bxdfs = BxDFHolder::new(arena);

        if let Some(ref bump_map) = self.bumpmap {
            super::bump(bump_map, si);
        }

        let e = self.eta.evaluate(si);
        let op = self.opacity.evaluate(si).clamp();
        let t = (Spectrum::white() - op).clamp();

        let mut eta = e;
        if !t.is_black() {
            eta = 1.0;
            bxdfs.add(arena.alloc(SpecularTransmission::new(t, 1.0, 1.0, mode)));
        }

        let kd = op * self.kd.evaluate(si).clamp();
        if !kd.is_black() {
            bxdfs.add(arena.alloc(LambertianReflection::new(kd)));
        }

        let ks = op * self.ks.evaluate(si).clamp();
        if !ks.is_black() {
            let fresnel = arena.alloc(dielectric(1.0, e));
            let mut roughu = self
                .roughnessu
                .as_ref()
                .unwrap_or(&self.roughness)
                .evaluate(si);
            let mut roughv = self
                .roughnessv
                .as_ref()
                .unwrap_or(&self.roughness)
                .evaluate(si);
            if self.remap_roughness {
                roughu = TrowbridgeReitzDistribution::roughness_to_alpha(roughu);
                roughv = TrowbridgeReitzDistribution::roughness_to_alpha(roughv);
            }
            let distrib = arena.alloc(TrowbridgeReitzDistribution::new(roughu, roughv));
            bxdfs.add(arena.alloc(MicrofacetReflection::new(ks, distrib, fresnel)));
        }

        let kr = op * self.kr.evaluate(si).clamp();
        if !kr.is_black() {
            let fresnel = arena.alloc(dielectric(1.0, e));
            bxdfs.add(arena.alloc(SpecularReflection::new(kr, fresnel)));
        }

        let kt = op * self.kt.evaluate(si).clamp();
        if !kt.is_black() {
            bxdfs.add(arena.alloc(SpecularTransmission::new(kt, 1.0, e, mode)));
        }

        let bsdf: Bsdf<'b> = Bsdf::new(si, eta, bxdfs.into_slice());
        si.bsdf = Some(Arc::new(bsdf));
    }
}
