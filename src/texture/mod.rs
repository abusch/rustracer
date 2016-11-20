use interaction::SurfaceInteraction;

pub trait Texture<T> {
    fn evaluate(&self, si: &SurfaceInteraction) -> T;
}

pub struct ConstantTexture<T> {
    value: T,
}

impl<T: Copy> ConstantTexture<T> {
    pub fn new(value: T) -> ConstantTexture<T> {
        ConstantTexture { value: value }
    }
}

impl<T: Copy> Texture<T> for ConstantTexture<T> {
    fn evaluate(&self, si: &SurfaceInteraction) -> T {
        self.value
    }
}
