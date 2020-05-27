use std::fmt::Debug;

use crate::interaction::SurfaceInteraction;
use crate::paramset::TextureParams;
use crate::spectrum::Spectrum;
use crate::texture::Texture;
use crate::Transform;

#[derive(Debug)]
pub struct ConstantTexture<T> {
    value: T,
}

impl<T: Copy> ConstantTexture<T> {
    pub fn new(value: T) -> ConstantTexture<T> {
        ConstantTexture { value }
    }
}

impl ConstantTexture<f32> {
    pub fn create_float(_tex2world: &Transform, tp: &TextureParams<'_>) -> ConstantTexture<f32> {
        ConstantTexture::new(tp.find_float("value", 1.0))
    }
}

impl ConstantTexture<Spectrum> {
    pub fn create_spectrum(
        _tex2world: &Transform,
        tp: &TextureParams<'_>,
    ) -> ConstantTexture<Spectrum> {
        ConstantTexture::new(tp.find_spectrum("value", Spectrum::white()))
    }
}

impl<T: Copy + Debug + Send + Sync> Texture<T> for ConstantTexture<T> {
    fn evaluate(&self, _si: &SurfaceInteraction<'_, '_>) -> T {
        self.value
    }
}
