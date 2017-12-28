use std::marker::PhantomData;

use Transform;
use interaction::SurfaceInteraction;
use paramset::TextureParams;
use noise;
use spectrum::Spectrum;
use texture::{Texture, TextureMapping3D, IdentityMapping3D};

#[derive(Debug)]
pub struct FbmTexture<T> {
    mapping: Box<TextureMapping3D>,
    roughness: f32,
    octaves: u32,
    _phantom: PhantomData<T>,
}

impl<T> FbmTexture<T> {
    fn evaluate_as_float(&self, si: &SurfaceInteraction) -> f32 {
        let (p, dpdx, dpdy) = self.mapping.map(si);
        noise::fbm(&p, &dpdx, &dpdy, self.roughness, self.octaves)
    }
}

impl FbmTexture<f32> {
    pub fn create_float(tex2world: &Transform, tp: &mut TextureParams) -> FbmTexture<f32> {
        FbmTexture {
            mapping: Box::new(IdentityMapping3D::new(tex2world.clone())),
            roughness: tp.find_float("omega", 0.5),
            octaves: tp.find_int("octaves", 8) as u32,
            _phantom: PhantomData,
        }
    }
}

impl FbmTexture<Spectrum> {
    pub fn create_spectrum(tex2world: &Transform, tp: &mut TextureParams) -> FbmTexture<Spectrum> {
        FbmTexture {
            mapping: Box::new(IdentityMapping3D::new(tex2world.clone())),
            roughness: tp.find_float("omega", 0.5),
            octaves: tp.find_int("octaves", 8) as u32,
            _phantom: PhantomData,
        }
    }
}

impl Texture<f32> for FbmTexture<f32> {
    fn evaluate(&self, si: &SurfaceInteraction) -> f32 {
        self.evaluate_as_float(si)
    }
}

impl Texture<Spectrum> for FbmTexture<Spectrum> {
    fn evaluate(&self, si: &SurfaceInteraction) -> Spectrum {
        Spectrum::from(self.evaluate_as_float(si))
    }
}
