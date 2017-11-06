use std::sync::Arc;

use light_arena::Allocator;

use bsdf::{BxDF, Fresnel, SpecularReflection, BSDF};
use interaction::SurfaceInteraction;
use material::{Material, TransportMode};
use paramset::TextureParams;
use texture::Texture;
use spectrum::Spectrum;

#[derive(Debug)]
pub struct MirrorMaterial {
    kr: Arc<Texture<Spectrum> + Send + Sync>,
    bump_map: Option<Arc<Texture<f32> + Send + Sync>>,
}

impl MirrorMaterial {
    pub fn create(mp: &mut TextureParams) -> Arc<Material + Sync + Send> {
        info!("Creating Mirror material");
        let Kr = mp.get_spectrum_texture("Kr", &Spectrum::grey(0.9));
        let bump_map = mp.get_float_texture_or_none("bumpmap");

        Arc::new(MirrorMaterial { kr: Kr, bump_map })
    }
}

impl Material for MirrorMaterial {
    fn compute_scattering_functions<'a, 'b>(&self,
                                            si: &mut SurfaceInteraction<'a, 'b>,
                                            _mode: TransportMode,
                                            _allow_multiple_lobes: bool,
                                            arena: &'b Allocator) {
        if let Some(ref bump) = self.bump_map {
            super::bump(bump, si);
        }
        let mut bxdfs = arena.alloc_slice::<&BxDF>(8);
        let mut i = 0;
        let R = self.kr.evaluate(si); // TODO clamp
        if !R.is_black() {
            let fresnel = arena <- Fresnel::no_op();
            bxdfs[i] = arena <- SpecularReflection::new(R, fresnel);
            i += 1;
        }

        unsafe {
            let ptr = bxdfs.as_mut_ptr();
            bxdfs = ::std::slice::from_raw_parts_mut(ptr, i);
        }

        si.bsdf = Some(Arc::new(BSDF::new(si, 1.0, bxdfs)));
    }
}
