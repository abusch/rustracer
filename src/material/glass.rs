use std::sync::Arc;

use bsdf::{BSDF, BxDF, FresnelSpecular, SpecularTransmission, SpecularReflection, Fresnel,
           MicrofacetReflection, MicrofacetTransmission, TrowbridgeReitzDistribution};
use interaction::SurfaceInteraction;
use material::{Material, TransportMode};
use spectrum::Spectrum;
use texture::{Texture, ConstantTexture};

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
}


impl Material for GlassMaterial {
    fn compute_scattering_functions(&self,
                                    isect: &mut SurfaceInteraction,
                                    mode: TransportMode,
                                    allow_multiple_lobes: bool) {
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
                    if is_specular {
                        let bxdf = Box::new(SpecularReflection::new(r, fresnel));
                        bxdfs.push(bxdf);
                    } else {
                        let distrib = Box::new(TrowbridgeReitzDistribution::new(u_rough, v_rough));
                        let bxdf = Box::new(MicrofacetReflection::new(r, distrib, fresnel));
                    }
                }
                if !t.is_black() {
                    if is_specular {
                        let bxdf = Box::new(SpecularTransmission::new(t, 1.0, eta));
                        bxdfs.push(bxdf);
                    } else {
                        let distrib = Box::new(TrowbridgeReitzDistribution::new(u_rough, v_rough));
                        let bxdf = Box::new(MicrofacetTransmission::new(r, distrib, 1.0, eta));
                    }
                }
            }
        }

        isect.bsdf = Some(Arc::new(BSDF::new(isect, eta, bxdfs)));
    }
}
