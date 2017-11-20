use std::sync::Arc;

use light_arena::Allocator;

use bsdf::{BxDFHolder, LambertianReflection, OrenNayar, BSDF};
use interaction::SurfaceInteraction;
use material::{Material, TransportMode};
use paramset::TextureParams;
use spectrum::Spectrum;
use texture::Texture;

#[derive(Debug)]
pub struct MatteMaterial {
    kd: Arc<Texture<Spectrum> + Sync + Send>,
    sigma: Arc<Texture<f32> + Sync + Send>,
    bump_map: Option<Arc<Texture<f32> + Sync + Send>>,
}

impl MatteMaterial {
    pub fn create(mp: &mut TextureParams) -> Arc<Material + Send + Sync> {
        info!("Creating Matte material");
        let kd = mp.get_spectrum_texture("Kd", &Spectrum::grey(0.5));
        let sigma = mp.get_float_texture("sigma", 0.0);
        let bump_map = mp.get_float_texture_or_none("bumpmap");

        Arc::new(MatteMaterial {
                     kd,
                     sigma,
                     bump_map,
                 })
    }
}

impl Material for MatteMaterial {
    fn compute_scattering_functions<'a, 'b>(&self,
                                            si: &mut SurfaceInteraction<'a, 'b>,
                                            _mode: TransportMode,
                                            _allow_multiple_lobes: bool,
                                            arena: &'b Allocator) {
        let mut bxdfs = BxDFHolder::new(arena);

        if let Some(ref bump_map) = self.bump_map {
            super::bump(bump_map, si);
        }

        let r = self.kd.evaluate(si);
        let sigma = self.sigma.evaluate(si);
        if sigma == 0.0 {
            bxdfs.add(arena <- LambertianReflection::new(r));
        } else {
            bxdfs.add(arena <- OrenNayar::new(r, sigma));
        }

        let bsdf = BSDF::new(si, 1.0, bxdfs.to_slice());
        si.bsdf = Some(Arc::new(bsdf));
    }
}
