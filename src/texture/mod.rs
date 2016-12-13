use ::Point2f;
use interaction::SurfaceInteraction;

mod constant;
mod checkerboard;
mod image;

pub use self::constant::ConstantTexture;
pub use self::checkerboard::CheckerboardTexture;
pub use self::image::ImageTexture;

pub trait Texture<T> {
    fn evaluate(&self, si: &SurfaceInteraction) -> T;
}

// Texture mappings

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
