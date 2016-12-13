use interaction::SurfaceInteraction;
use texture::Texture;

pub struct ConstantTexture<T> {
    value: T,
}

impl<T: Copy> ConstantTexture<T> {
    pub fn new(value: T) -> ConstantTexture<T> {
        ConstantTexture { value: value }
    }
}

impl<T: Copy> Texture<T> for ConstantTexture<T> {
    fn evaluate(&self, _si: &SurfaceInteraction) -> T {
        self.value
    }
}
