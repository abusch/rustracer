use std::sync::Arc;

use bsdf::{BSDF, BxDF, Fresnel, TrowbridgeReitzDistribution, MicrofacetReflection,
           LambertianReflection};
use spectrum::Spectrum;
use interaction::SurfaceInteraction;
use material::{Material, TransportMode};
use texture::{Texture, ConstantTexture};


pub struct Plastic {
    kd: Arc<Texture<Spectrum> + Send + Sync>,
    ks: Arc<Texture<Spectrum> + Send + Sync>, // TODO roughness, bump
}

impl Plastic {
    pub fn new(kd: Spectrum, ks: Spectrum) -> Plastic {
        Plastic {
            kd: Arc::new(ConstantTexture::new(kd)),
            ks: Arc::new(ConstantTexture::new(ks)),
        }
    }
}

impl Material for Plastic {
    fn compute_scattering_functions(&self,
                                    si: &mut SurfaceInteraction,
                                    _mode: TransportMode,
                                    _allow_multiple_lobes: bool) {
        let mut bxdfs: Vec<Box<BxDF + Send + Sync>> = Vec::new();
        let kd = self.kd.evaluate(si);
        if !kd.is_black() {
            bxdfs.push(Box::new(LambertianReflection::new(kd)));
        }
        let ks = self.ks.evaluate(si);
        if !ks.is_black() {
            let fresnel = Box::new(Fresnel::dielectric(1.0, 1.5));
            let roughness = TrowbridgeReitzDistribution::roughness_to_alpha(0.9);
            let distrib = Box::new(TrowbridgeReitzDistribution::new(roughness, roughness));
            bxdfs.push(Box::new(MicrofacetReflection::new(ks, distrib, fresnel)));
        }

        let bsdf = BSDF::new(si, 1.5, bxdfs);
        si.bsdf = Some(Arc::new(bsdf));
    }
}
