use interaction::SurfaceInteraction;
use noise;
use texture::{Texture, TextureMapping3D, TransformMapping3D};

#[derive(Debug)]
pub struct FbmTexture {
    mapping: Box<TextureMapping3D>,
    omega: f32,
    octaves: u32,
}

impl FbmTexture {
    pub fn new() -> FbmTexture {
        FbmTexture {
            mapping: Box::new(TransformMapping3D::new()),
            omega: 0.5,
            octaves: 8,
        }
    }
}

impl Texture<f32> for FbmTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> f32 {
        let (p, dpdx, dpdy) = self.mapping.map(si);
        noise::fbm(&p, &dpdx, &dpdy, self.omega, self.octaves)
    }
}
