use std::sync::Arc;

use light_arena::Allocator;

use bsdf::{BxDF, FresnelBlend, TrowbridgeReitzDistribution, BSDF};
use interaction::SurfaceInteraction;
use material::{Material, TransportMode};
use paramset::TextureParams;
use spectrum::Spectrum;
use texture::Texture;

#[derive(Debug)]
pub struct SubstrateMaterial {
    kd: Arc<Texture<Spectrum> + Send + Sync>,
    ks: Arc<Texture<Spectrum> + Send + Sync>,
    nu: Arc<Texture<f32> + Send + Sync>,
    nv: Arc<Texture<f32> + Send + Sync>,
    remap_roughness: bool,
}

impl SubstrateMaterial {
    pub fn create(mp: &mut TextureParams) -> Arc<Material + Send + Sync> {
        let kd = mp.get_spectrum_texture("Kd", &Spectrum::grey(0.5));
        let ks = mp.get_spectrum_texture("Ks", &Spectrum::grey(0.5));
        let urough = mp.get_float_texture("uroughness", 0.1);
        let vrough = mp.get_float_texture("vroughness", 0.1);
        let remap_roughness = mp.find_bool("remaproughness", true);

        Arc::new(SubstrateMaterial {
            kd,
            ks,
            nu: urough,
            nv: vrough,
            remap_roughness,
        })
    }
}

impl Material for SubstrateMaterial {
    fn compute_scattering_functions<'a, 'b>(
        &self,
        si: &mut SurfaceInteraction<'a, 'b>,
        _mode: TransportMode,
        _allow_multiple_lobes: bool,
        arena: &'b Allocator,
    ) {
        let mut bxdfs = arena.alloc_slice::<&BxDF>(8);
        let mut i = 0;

        let d = self.kd.evaluate(si);
        let s = self.ks.evaluate(si);
        let mut roughu = self.nu.evaluate(si);
        let mut roughv = self.nv.evaluate(si);

        if !d.is_black() || !s.is_black() {
            if self.remap_roughness {
                roughu = TrowbridgeReitzDistribution::roughness_to_alpha(roughu);
                roughv = TrowbridgeReitzDistribution::roughness_to_alpha(roughv);
            }
            let distrib = arena <- TrowbridgeReitzDistribution::new(roughu, roughv);
            bxdfs[i] = arena <- FresnelBlend::new(d, s, distrib);
            i += 1;
        }

        unsafe {
            let ptr = bxdfs.as_mut_ptr();
            bxdfs = ::std::slice::from_raw_parts_mut(ptr, i);
        }

        let bsdf = BSDF::new(si, 1.0, bxdfs);
        si.bsdf = Some(Arc::new(bsdf));
    }
}
