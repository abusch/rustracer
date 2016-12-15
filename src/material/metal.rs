use std::sync::Arc;

use bsdf::{BSDF, BxDF, Fresnel, TrowbridgeReitzDistribution, MicrofacetReflection};
use interaction::SurfaceInteraction;
use material::{Material, TransportMode};
use spectrum::Spectrum;
use texture::{Texture, ConstantTexture};

pub struct Metal {
    eta: Box<Texture<Spectrum> + Send + Sync>,
    k: Box<Texture<Spectrum> + Send + Sync>,
    rough: Box<Texture<f32> + Send + Sync>,
    // urough: Box<Texture<f32>>,
    // vrough: Box<Texture<f32>>,
    remap_roughness: bool,
}

impl Metal {
    pub fn new() -> Self {
        Metal {
            eta: Box::new(ConstantTexture::new(Spectrum::from_sampled(&CopperWavelengths[..],
                                                                      &CopperN[..],
                                                                      CopperSamples))),
            k: Box::new(ConstantTexture::new(Spectrum::from_sampled(&CopperWavelengths[..],
                                                                    &CopperK[..],
                                                                    CopperSamples))),
            rough: Box::new(ConstantTexture::new(0.1)),
            // urough: Box::new(ConstantTexture::new(...)),
            // vrough: Box::new(ConstantTexture::new(...)),
            remap_roughness: true,
        }
    }
}

impl Material for Metal {
    fn compute_scattering_functions(&self,
                                    si: &mut SurfaceInteraction,
                                    mode: TransportMode,
                                    allow_multiple_lobes: bool) {
        let mut bxdfs: Vec<Box<BxDF + Send + Sync>> = Vec::new();
        let mut rough = self.rough.evaluate(si);
        if self.remap_roughness {
            rough = TrowbridgeReitzDistribution::roughness_to_alpha(rough);
        }
        let fresnel = Box::new(Fresnel::conductor(Spectrum::white(),
                                                  self.eta.evaluate(si),
                                                  self.k.evaluate(si)));
        let distrib = Box::new(TrowbridgeReitzDistribution::new(rough, rough));
        bxdfs.push(Box::new(MicrofacetReflection::new(Spectrum::white(), distrib, fresnel)));

        let bsdf = BSDF::new(si, 1.0, bxdfs);
        si.bsdf = Some(Arc::new(bsdf));
    }
}

const CopperSamples: usize = 56;
const CopperWavelengths: [f32; CopperSamples] = [298.7570554,
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
                                                 885.6012714];

const CopperN: [f32; CopperSamples] =
    [1.400313, 1.38, 1.358438, 1.34, 1.329063, 1.325, 1.3325, 1.34, 1.334375, 1.325, 1.317812,
     1.31, 1.300313, 1.29, 1.281563, 1.27, 1.249062, 1.225, 1.2, 1.18, 1.174375, 1.175, 1.1775,
     1.18, 1.178125, 1.175, 1.172812, 1.17, 1.165312, 1.16, 1.155312, 1.15, 1.142812, 1.135,
     1.131562, 1.12, 1.092437, 1.04, 0.950375, 0.826, 0.645875, 0.468, 0.35125, 0.272, 0.230813,
     0.214, 0.20925, 0.213, 0.21625, 0.223, 0.2365, 0.25, 0.254188, 0.26, 0.28, 0.3];

const CopperK: [f32; CopperSamples] =
    [1.662125, 1.687, 1.703313, 1.72, 1.744563, 1.77, 1.791625, 1.81, 1.822125, 1.834, 1.85175,
     1.872, 1.89425, 1.916, 1.931688, 1.95, 1.972438, 2.015, 2.121562, 2.21, 2.177188, 2.13,
     2.160063, 2.21, 2.249938, 2.289, 2.326, 2.362, 2.397625, 2.433, 2.469187, 2.504, 2.535875,
     2.564, 2.589625, 2.605, 2.595562, 2.583, 2.5765, 2.599, 2.678062, 2.809, 3.01075, 3.24,
     3.458187, 3.67, 3.863125, 4.05, 4.239563, 4.43, 4.619563, 4.817, 5.034125, 5.26, 5.485625,
     5.717];
