use std::sync::Arc;


use ::Point2f;
use colour::Colourf;
use interaction::SurfaceInteraction;

pub trait Texture<T> {
    fn evaluate(&self, si: &SurfaceInteraction) -> T;
}

// Constant texture
//

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

// Checkerboard texture
//

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

impl CheckerboardTexture<Colourf> {
    pub fn bw() -> CheckerboardTexture<Colourf> {
        CheckerboardTexture::new(Arc::new(ConstantTexture::new(Colourf::black())),
                                 Arc::new(ConstantTexture::new(Colourf::white())),
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

// Texture Mappings
//

pub trait TextureMapping2D {
    fn map(&self, si: &SurfaceInteraction) -> Point2f;
}

pub struct UVMapping2D {
    su: f32,
    sv: f32,
    du: f32,
    dv: f32,
}

impl UVMapping2D {
    pub fn new(su: f32, sv: f32, du: f32, dv: f32) -> UVMapping2D {
        UVMapping2D {
            su: su,
            sv: sv,
            du: du,
            dv: dv,
        }
    }
}

impl TextureMapping2D for UVMapping2D {
    fn map(&self, si: &SurfaceInteraction) -> Point2f {
        Point2f::new(self.su * si.uv.x + self.du, self.sv * si.uv.y + self.dv)
    }
}
