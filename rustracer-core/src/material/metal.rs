use std::sync::Arc;

use light_arena::Allocator;

use crate::bsdf::{conductor, BxDFHolder, MicrofacetReflection, TrowbridgeReitzDistribution, BSDF};
use crate::interaction::SurfaceInteraction;
use crate::material::{self, Material, TransportMode};
use crate::paramset::TextureParams;
use crate::spectrum::Spectrum;
use crate::texture::{TextureFloat, TextureSpectrum};

#[derive(Debug)]
pub struct Metal {
    eta: Arc<TextureSpectrum>,
    k: Arc<TextureSpectrum>,
    rough: Arc<TextureFloat>,
    bump: Option<Arc<TextureFloat>>,
    urough: Option<Arc<TextureFloat>>,
    vrough: Option<Arc<TextureFloat>>,
    remap_roughness: bool,
}

impl Metal {
    pub fn create(mp: &TextureParams<'_>) -> Arc<dyn Material> {
        let copper_eta =
            Spectrum::from_sampled(&COPPER_WAVELENGTHS[..], &COPPER_N[..], COPPER_SAMPLES);
        let eta = mp.get_spectrum_texture("eta", &copper_eta);
        let copper_k =
            Spectrum::from_sampled(&COPPER_WAVELENGTHS[..], &COPPER_K[..], COPPER_SAMPLES);
        let k = mp.get_spectrum_texture("k", &copper_k);
        let rough = mp.get_float_texture("roughness", 0.01);
        let urough = mp.get_float_texture_or_none("uroughness");
        let vrough = mp.get_float_texture_or_none("vroughness");
        let bump = mp.get_float_texture_or_none("bumpmap");
        let remap_roughness = mp.find_bool("remaproughness", true);

        Arc::new(Metal {
            eta,
            k,
            rough,
            bump,
            urough,
            vrough,
            remap_roughness,
        })
    }
}

impl Material for Metal {
    fn compute_scattering_functions<'a, 'b>(
        &self,
        si: &mut SurfaceInteraction<'a, 'b>,
        _mode: TransportMode,
        _allow_multiple_lobes: bool,
        arena: &'b Allocator<'_>,
    ) {
        if let Some(ref bump) = self.bump {
            material::bump(bump, si);
        }
        let mut bxdfs = BxDFHolder::new(arena);
        let mut urough = self.urough.as_ref().unwrap_or(&self.rough).evaluate(si);
        let mut vrough = self.vrough.as_ref().unwrap_or(&self.rough).evaluate(si);
        if self.remap_roughness {
            urough = TrowbridgeReitzDistribution::roughness_to_alpha(urough);
            vrough = TrowbridgeReitzDistribution::roughness_to_alpha(vrough);
        }
        let fresnel = arena.alloc(conductor(
            Spectrum::white(),
            self.eta.evaluate(si),
            self.k.evaluate(si),
        ));
        let distrib = arena.alloc(TrowbridgeReitzDistribution::new(urough, vrough));
        bxdfs.add(arena.alloc(MicrofacetReflection::new(
            Spectrum::white(),
            distrib,
            fresnel,
        )));

        let bsdf = BSDF::new(si, 1.0, bxdfs.into_slice());
        si.bsdf = Some(Arc::new(bsdf));
    }
}

const COPPER_SAMPLES: usize = 56;
const COPPER_WAVELENGTHS: [f32; COPPER_SAMPLES] = [
    298.7570554,
    302.4004341,
    306.1337728,
    309.960445,
    313.8839949,
    317.9081487,
    322.036826,
    326.2741526,
    330.6244747,
    335.092373,
    339.6826795,
    344.4004944,
    349.2512056,
    354.2405086,
    359.374429,
    364.6593471,
    370.1020239,
    375.7096303,
    381.4897785,
    387.4505563,
    393.6005651,
    399.9489613,
    406.5055016,
    413.2805933,
    420.2853492,
    427.5316483,
    435.0322035,
    442.8006357,
    450.8515564,
    459.2006593,
    467.8648226,
    476.8622231,
    486.2124627,
    495.936712,
    506.0578694,
    516.6007417,
    527.5922468,
    539.0616435,
    551.0407911,
    563.5644455,
    576.6705953,
    590.4008476,
    604.8008683,
    619.92089,
    635.8162974,
    652.5483053,
    670.1847459,
    688.8009889,
    708.4810171,
    729.3186941,
    751.4192606,
    774.9011125,
    799.8979226,
    826.5611867,
    855.0632966,
    885.6012714,
];

const COPPER_N: [f32; COPPER_SAMPLES] = [
    1.400313, 1.38, 1.358438, 1.34, 1.329063, 1.325, 1.3325, 1.34, 1.334375, 1.325, 1.317812, 1.31,
    1.300313, 1.29, 1.281563, 1.27, 1.249062, 1.225, 1.2, 1.18, 1.174375, 1.175, 1.1775, 1.18,
    1.178125, 1.175, 1.172812, 1.17, 1.165312, 1.16, 1.155312, 1.15, 1.142812, 1.135, 1.131562,
    1.12, 1.092437, 1.04, 0.950375, 0.826, 0.645875, 0.468, 0.35125, 0.272, 0.230813, 0.214,
    0.20925, 0.213, 0.21625, 0.223, 0.2365, 0.25, 0.254188, 0.26, 0.28, 0.3,
];

const COPPER_K: [f32; COPPER_SAMPLES] = [
    1.662125, 1.687, 1.703313, 1.72, 1.744563, 1.77, 1.791625, 1.81, 1.822125, 1.834, 1.85175,
    1.872, 1.89425, 1.916, 1.931688, 1.95, 1.972438, 2.015, 2.121562, 2.21, 2.177188, 2.13,
    2.160063, 2.21, 2.249938, 2.289, 2.326, 2.362, 2.397625, 2.433, 2.469187, 2.504, 2.535875,
    2.564, 2.589625, 2.605, 2.595562, 2.583, 2.5765, 2.599, 2.678062, 2.809, 3.01075, 3.24,
    3.458187, 3.67, 3.863125, 4.05, 4.239563, 4.43, 4.619563, 4.817, 5.034125, 5.26, 5.485625,
    5.717,
];
