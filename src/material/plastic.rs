use std::sync::Arc;

use light_arena::Allocator;

use bsdf::{BxDF, Fresnel, LambertianReflection, MicrofacetReflection, TrowbridgeReitzDistribution,
           BSDF};
use spectrum::Spectrum;
use interaction::SurfaceInteraction;
use material::{Material, TransportMode};
use paramset::TextureParams;
use texture::Texture;


#[derive(Debug)]
pub struct Plastic {
    kd: Arc<Texture<Spectrum> + Send + Sync>,
    ks: Arc<Texture<Spectrum> + Send + Sync>,
    roughness: Arc<Texture<f32> + Send + Sync>,
    remap_roughness: bool, // TODO bump
}

impl Plastic {
    pub fn create(mp: &mut TextureParams) -> Arc<Material + Send + Sync> {
        info!("Creating Plastic material");
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
    fn compute_scattering_functions<'a, 'b>(
        &self,
        si: &mut SurfaceInteraction<'a, 'b>,
        _mode: TransportMode,
        _allow_multiple_lobes: bool,
        arena: &'b Allocator,
    ) {
        let kd = self.kd.evaluate(si);
        let ks = self.ks.evaluate(si);

        let mut bxdfs = arena.alloc_slice::<&BxDF>(8);
        let mut i = 0;
        if !kd.is_black() {
            bxdfs[i] = arena <- LambertianReflection::new(kd);
            i += 1;
        }
        if !ks.is_black() {
            let fresnel = arena <- Fresnel::dielectric(1.5, 1.0);
            let mut roughness = self.roughness.evaluate(si);
            if self.remap_roughness {
                roughness = TrowbridgeReitzDistribution::roughness_to_alpha(roughness);
            }
            let distrib = arena <- TrowbridgeReitzDistribution::new(roughness, roughness);
            bxdfs[i] = arena <- MicrofacetReflection::new(ks, distrib, fresnel);
            i += 1;
        }

        unsafe {
            let ptr = bxdfs.as_mut_ptr();
            bxdfs = ::std::slice::from_raw_parts_mut(ptr, i);
        }

        let bsdf: BSDF<'b> = BSDF::new(si, 1.0, bxdfs);
        si.bsdf = Some(Arc::new(bsdf));
    }
}
