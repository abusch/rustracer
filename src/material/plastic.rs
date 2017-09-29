use std::sync::Arc;
use std::path::Path;

use bsdf::{BxDF, Fresnel, LambertianReflection, MicrofacetReflection, TrowbridgeReitzDistribution,
           BSDF};
use spectrum::Spectrum;
use interaction::SurfaceInteraction;
use material::{Material, TransportMode};
use paramset::TextureParams;
use texture::{ConstantTexture, ImageTexture, Texture};


pub struct Plastic {
    kd: Arc<Texture<Spectrum> + Send + Sync>,
    ks: Arc<Texture<Spectrum> + Send + Sync>,
    roughness: Arc<Texture<f32> + Send + Sync>,
    remap_roughness: bool, // TODO bump
}

impl Plastic {
    pub fn new(kd: Spectrum, ks: Spectrum) -> Plastic {
        Plastic {
            kd: Arc::new(ConstantTexture::new(kd)),
            ks: Arc::new(ConstantTexture::new(ks)),
            roughness: Arc::new(ConstantTexture::new(0.1)),
            remap_roughness: true,
        }
    }

    pub fn new_tex(kd_tex: &str, ks: Spectrum) -> Plastic {
        Plastic {
            kd: Arc::new(ImageTexture::new(Path::new(kd_tex))),
            ks: Arc::new(ConstantTexture::new(ks)),
            roughness: Arc::new(ConstantTexture::new(0.1)),
            remap_roughness: true,
        }
    }

    pub fn create(mp: &mut TextureParams) -> Arc<Material + Send + Sync> {
        let Kd = mp.get_spectrum_texture("Kd", &Spectrum::grey(0.25));
        let Ks = mp.get_spectrum_texture("Ks", &Spectrum::grey(0.25));
        let roughness = mp.get_float_texture("roughness", 0.1);
        let remap_roughness = mp.find_bool("remaproughness", true);

        Arc::new(Plastic {
            kd: Kd,
            ks: Ks,
            roughness: roughness,
            remap_roughness: remap_roughness,
        })
    }
}

impl Material for Plastic {
    fn compute_scattering_functions(
        &self,
        si: &mut SurfaceInteraction,
        _mode: TransportMode,
        _allow_multiple_lobes: bool,
    ) {
        let mut bxdfs: Vec<Box<BxDF + Send + Sync>> = Vec::new();
        let kd = self.kd.evaluate(si);
        if !kd.is_black() {
            bxdfs.push(Box::new(LambertianReflection::new(kd)));
        }
        let ks = self.ks.evaluate(si);
        if !ks.is_black() {
            let fresnel = Box::new(Fresnel::dielectric(1.5, 1.0));
            let mut roughness = self.roughness.evaluate(si);
            if self.remap_roughness {
                roughness = TrowbridgeReitzDistribution::roughness_to_alpha(roughness);
            }
            let distrib = Box::new(TrowbridgeReitzDistribution::new(roughness, roughness));
            bxdfs.push(Box::new(MicrofacetReflection::new(ks, distrib, fresnel)));
        }

        let bsdf = BSDF::new(si, 1.5, bxdfs);
        si.bsdf = Some(Arc::new(bsdf));
    }
}
