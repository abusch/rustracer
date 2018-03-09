use std::sync::Arc;

use light_arena::Allocator;

use bsdf::{BxDFHolder, FourierBSDF, FourierBSDFTable, BSDF};
use interaction::SurfaceInteraction;
use material::{Material, TransportMode};
use paramset::TextureParams;
use texture::TextureFloat;

#[derive(Debug)]
pub struct FourierMaterial {
    bsdf_table: Box<FourierBSDFTable>,
    bump_map: Option<Arc<TextureFloat>>,
}

impl FourierMaterial {
    pub fn create(mp: &TextureParams) -> Arc<dyn Material> {
        let bump_map = mp.get_float_texture_or_none("bumpmap");
        let filename = mp.find_filename("bsdffile", "");
        let bsdf_table = Box::new(FourierBSDFTable::read(filename).unwrap()); // TODO error
        Arc::new(FourierMaterial {
            bsdf_table,
            bump_map,
        })
    }
}

impl Material for FourierMaterial {
    fn compute_scattering_functions<'a, 'b>(
        &self,
        si: &mut SurfaceInteraction<'a, 'b>,
        mode: TransportMode,
        _allow_multiple_lobes: bool,
        arena: &'b Allocator,
    ) {
        let mut bxdfs = BxDFHolder::new(arena);

        if let Some(ref bump) = self.bump_map {
            super::bump(bump, si);
        }
        bxdfs.add(arena <- FourierBSDF::new(&self.bsdf_table, mode));
        let bsdf = BSDF::new(si, 1.0, bxdfs.into_slice());
        si.bsdf = Some(Arc::new(bsdf));
    }
}
