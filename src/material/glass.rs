use std::sync::Arc;

use bsdf::{BxDF, Fresnel, FresnelSpecular, MicrofacetReflection, MicrofacetTransmission,
           SpecularReflection, SpecularTransmission, TrowbridgeReitzDistribution, BSDF};
use interaction::SurfaceInteraction;
use paramset::{ParamSet, TextureParams};
use material::{Material, TransportMode};
use spectrum::Spectrum;
use texture::{ConstantTexture, Texture};

pub struct GlassMaterial {
    kr: Arc<Texture<Spectrum> + Send + Sync>,
    kt: Arc<Texture<Spectrum> + Send + Sync>,
    u_roughness: Arc<Texture<f32> + Send + Sync>,
    v_roughness: Arc<Texture<f32> + Send + Sync>,
    index: Arc<Texture<f32> + Send + Sync>,
    remap_roughness: bool,
}

impl GlassMaterial {
    pub fn new() -> GlassMaterial {
        GlassMaterial {
            kr: Arc::new(ConstantTexture::new(Spectrum::white())),
            kt: Arc::new(ConstantTexture::new(Spectrum::white())),
            u_roughness: Arc::new(ConstantTexture::new(0.0)),
            v_roughness: Arc::new(ConstantTexture::new(0.0)),
            index: Arc::new(ConstantTexture::new(1.5)),
            remap_roughness: true,
        }
    }

    pub fn roughness(mut self, u_rough: f32, v_rough: f32) -> GlassMaterial {
        self.u_roughness = Arc::new(ConstantTexture::new(::na::clamp(u_rough, 0.0, 1.0)));
        self.v_roughness = Arc::new(ConstantTexture::new(::na::clamp(v_rough, 0.0, 1.0)));

        self
    }

    pub fn create(mp: &mut TextureParams) -> Arc<Material + Send + Sync> {
        let Kr = mp.get_spectrum_texture("Kr", &Spectrum::white());
        let Kt = mp.get_spectrum_texture("Kt", &Spectrum::white());
        let eta = mp.get_float_texture_or_none("eta")
            .unwrap_or_else(|| mp.get_float_texture("index", 1.5));
        let rough_u = mp.get_float_texture("uroughness", 0.0);
        let rough_v = mp.get_float_texture("vroughness", 0.0);
        // TODO bumpmap
        let remap_roughness = mp.find_bool("remaproughness", true);

        Arc::new(GlassMaterial {
            kr: Kr,
            kt: Kt,
            u_roughness: rough_u,
            v_roughness: rough_v,
            index: eta,
            remap_roughness,
        })
    }
}


impl Material for GlassMaterial {
    fn compute_scattering_functions(
        &self,
        isect: &mut SurfaceInteraction,
        _mode: TransportMode,
        allow_multiple_lobes: bool,
    ) {
        let eta = self.index.evaluate(isect);
        let mut u_rough = self.u_roughness.evaluate(isect);
        let mut v_rough = self.v_roughness.evaluate(isect);
        let r = self.kr.evaluate(isect);
        let t = self.kt.evaluate(isect);

        let mut bxdfs: Vec<Box<BxDF + Send + Sync>> = Vec::new();

        if !r.is_black() || !t.is_black() {
            let is_specular = u_rough == 0.0 && v_rough == 0.0;
            if is_specular && allow_multiple_lobes {
                bxdfs.push(Box::new(FresnelSpecular::new()));
            } else {
                if self.remap_roughness {
                    u_rough = TrowbridgeReitzDistribution::roughness_to_alpha(u_rough);
                    v_rough = TrowbridgeReitzDistribution::roughness_to_alpha(v_rough);
                }
                if !r.is_black() {
                    let fresnel = Box::new(Fresnel::dielectric(1.0, eta));
                    let bxdf: Box<BxDF + Send + Sync> = if is_specular {
                        Box::new(SpecularReflection::new(r, fresnel))
                    } else {
                        let distrib = Box::new(TrowbridgeReitzDistribution::new(u_rough, v_rough));
                        Box::new(MicrofacetReflection::new(r, distrib, fresnel))
                    };
                    bxdfs.push(bxdf);
                }
                if !t.is_black() {
                    let bxdf: Box<BxDF + Send + Sync> = if is_specular {
                        Box::new(SpecularTransmission::new(t, 1.0, eta))
                    } else {
                        let distrib = Box::new(TrowbridgeReitzDistribution::new(u_rough, v_rough));
                        Box::new(MicrofacetTransmission::new(r, distrib, 1.0, eta))
                    };
                    bxdfs.push(bxdf);
                }
            }
        }

        isect.bsdf = Some(Arc::new(BSDF::new(isect, eta, bxdfs)));
    }
}
