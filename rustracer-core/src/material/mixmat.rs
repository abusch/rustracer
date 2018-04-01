use std::sync::Arc;

use light_arena::Allocator;

use bsdf::{BxDFHolder, ScaledBxDF};
use interaction::SurfaceInteraction;
use material::{Material, TransportMode};
use paramset::TextureParams;
use spectrum::Spectrum;
use texture::TextureSpectrum;

#[derive(Debug)]
pub struct MixMaterial {
    mat1: Arc<dyn Material>,
    mat2: Arc<dyn Material>,
    scale: Arc<TextureSpectrum>,
}

impl MixMaterial {
    pub fn create(
        mp: &TextureParams,
        m1: Arc<dyn Material>,
        m2: Arc<dyn Material>,
    ) -> Arc<dyn Material> {
        Arc::new(MixMaterial {
            mat1: m1,
            mat2: m2,
            scale: mp.get_spectrum_texture("amount", &Spectrum::from(0.5)),
        })
    }
}

impl Material for MixMaterial {
    fn compute_scattering_functions<'a, 'b>(
        &self,
        si: &mut SurfaceInteraction<'a, 'b>,
        mode: TransportMode,
        allow_multiple_lobes: bool,
        arena: &'b Allocator,
    ) {
        let s1 = self.scale.evaluate(si).clamp();
        let s2 = (Spectrum::white() - s1).clamp();
        let mut si2 = si.clone();
        self.mat1
            .compute_scattering_functions(si, mode, allow_multiple_lobes, arena);
        self.mat2
            .compute_scattering_functions(&mut si2, mode, allow_multiple_lobes, arena);
        let n1 = si.bsdf.as_ref().unwrap().bxdfs.len();
        let n2 = si2.bsdf.as_ref().unwrap().bxdfs.len();

        let mut bxdfs = BxDFHolder::new(arena);
        for i in 0..n1 {
            bxdfs.add(arena <- ScaledBxDF::new(si.bsdf.as_ref().unwrap().bxdfs[i], s1));
        }
        for i in 0..n2 {
            bxdfs.add(arena <- ScaledBxDF::new(si2.bsdf.as_ref().unwrap().bxdfs[i], s2));
        }

        si.bsdf.as_mut().map(|b| {
            (Arc::get_mut(b))
                .as_mut()
                .map(|b| b.bxdfs = bxdfs.into_slice())
        });
    }
}
