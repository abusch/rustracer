use std::fmt::Debug;

use Transform;
use interaction::SurfaceInteraction;
use paramset::TextureParams;
use spectrum::Spectrum;
use texture::Texture;

#[derive(Debug)]
pub struct ConstantTexture<T> {
    value: T,
}

impl<T: Copy> ConstantTexture<T> {
    pub fn new(value: T) -> ConstantTexture<T> {
        ConstantTexture { value: value }
    }
}

impl ConstantTexture<f32> {
    pub fn create_float(_tex2world: &Transform, tp: &mut TextureParams) -> ConstantTexture<f32> {
        ConstantTexture::new(tp.find_float("value", 1.0))
    }
}

impl ConstantTexture<Spectrum> {
    pub fn create_spectrum(_tex2world: &Transform,
                           tp: &mut TextureParams)
                           -> ConstantTexture<Spectrum> {
        ConstantTexture::new(tp.find_spectrum("value", Spectrum::white()))
    }
}

impl<T: Copy + Debug> Texture<T> for ConstantTexture<T> {
    fn evaluate(&self, _si: &SurfaceInteraction) -> T {
        self.value
    }
}
