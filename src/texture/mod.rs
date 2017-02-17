use na;

use {Point2f, Point3f, Vector3f, Transform};
use interaction::SurfaceInteraction;
use spectrum::Spectrum;

mod constant;
mod checkerboard;
mod imagemap;
mod fbm;

pub use self::constant::ConstantTexture;
pub use self::checkerboard::CheckerboardTexture;
pub use self::imagemap::ImageTexture;
pub use self::fbm::FbmTexture;

pub trait Texture<T> {
    fn evaluate(&self, si: &SurfaceInteraction) -> T;
}

pub struct UVTexture {
    mapping: Box<TextureMapping2D + Send + Sync>,
}

impl UVTexture {
    pub fn new() -> UVTexture {
        UVTexture { mapping: Box::new(UVMapping2D::new(1.0, 1.0, 0.0, 0.0)) }
    }
}

impl Texture<Spectrum> for UVTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> Spectrum {
        let st = self.mapping.map(si);
        Spectrum::rgb(st[0] - st[0].floor(), st[1] - st[1].floor(), 0.0)
    }
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

pub trait TextureMapping3D {
    fn map(&self, si: &SurfaceInteraction) -> (Point3f, Vector3f, Vector3f);
}

pub struct TransformMapping3D {
    world_to_texture: Transform,
}

impl TransformMapping3D {
    pub fn new() -> TransformMapping3D {
        TransformMapping3D { world_to_texture: na::one() }
    }
}

impl TextureMapping3D for TransformMapping3D {
    fn map(&self, si: &SurfaceInteraction) -> (Point3f, Vector3f, Vector3f) {
        let dpdx = self.world_to_texture * si.dpdx;
        let dpdy = self.world_to_texture * si.dpdy;
        let p = self.world_to_texture * si.p;

        (p, dpdx, dpdy)
    }
}
