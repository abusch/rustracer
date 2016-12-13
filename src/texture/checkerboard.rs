use std::sync::Arc;

use interaction::SurfaceInteraction;
use spectrum::Spectrum;
use texture::{Texture, TextureMapping2D, UVMapping2D, ConstantTexture};

pub struct CheckerboardTexture<T> {
    tex1: Arc<Texture<T> + Send + Sync>,
    tex2: Arc<Texture<T> + Send + Sync>,
    mapping: Box<TextureMapping2D + Send + Sync>,
}

impl<T> CheckerboardTexture<T> {
    pub fn new(tex1: Arc<Texture<T> + Send + Sync>,
               tex2: Arc<Texture<T> + Send + Sync>,
               mapping: Box<TextureMapping2D + Send + Sync>)
               -> CheckerboardTexture<T> {
        CheckerboardTexture {
            tex1: tex1,
            tex2: tex2,
            mapping: mapping,
        }
    }
}

impl CheckerboardTexture<Spectrum> {
    pub fn bw() -> CheckerboardTexture<Spectrum> {
        CheckerboardTexture::new(Arc::new(ConstantTexture::new(Spectrum::black())),
                                 Arc::new(ConstantTexture::new(Spectrum::white())),
                                 Box::new(UVMapping2D::new(10.0, 10.0, 0.0, 0.0)))
    }
}

impl<T> Texture<T> for CheckerboardTexture<T> {
    fn evaluate(&self, si: &SurfaceInteraction) -> T {
        let st = self.mapping.map(si);
        if (st.x.floor() as u32 + st.y.floor() as u32) % 2 == 0 {
            self.tex1.evaluate(si)
        } else {
            self.tex2.evaluate(si)
        }
    }
}
