use std::fmt::Debug;

use interaction::SurfaceInteraction;
use spectrum::Spectrum;
use {Point2f, Point3f, Transform, Vector2f, Vector3f};

mod checkerboard;
mod constant;
mod fbm;
mod imagemap;
mod mix;
mod scale;
mod uv;

pub use self::checkerboard::CheckerboardTexture;
pub use self::constant::ConstantTexture;
pub use self::fbm::FbmTexture;
pub use self::imagemap::ImageTexture;
pub use self::mix::MixTexture;
pub use self::scale::ScaleTexture;
pub use self::uv::UVTexture;

pub trait Texture<T>: Debug + Send + Sync {
    fn evaluate(&self, si: &SurfaceInteraction) -> T;
}

// Some convenient aliases
pub type TextureSpectrum = dyn Texture<Spectrum>;
pub type TextureFloat = dyn Texture<f32>;

// Texture mappings

pub trait TextureMapping2D: Debug + Send + Sync {
    fn map(&self, si: &SurfaceInteraction) -> (Point2f, Vector2f, Vector2f);
}

#[derive(Debug)]
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
    fn map(&self, si: &SurfaceInteraction) -> (Point2f, Vector2f, Vector2f) {
        (
            Point2f::new(self.su * si.uv.x + self.du, self.sv * si.uv.y + self.dv),
            // dstdx
            Vector2f::new(self.su * si.dudx, self.sv * si.dvdx),
            // dstdy
            Vector2f::new(self.su * si.dudy, self.sv * si.dvdy),
        )
    }
}

#[derive(Debug)]
struct PlanarMapping2D {
    vs: Vector3f,
    vt: Vector3f,
    ds: f32,
    dt: f32,
}

impl PlanarMapping2D {
    pub fn new(vs: Vector3f, vt: Vector3f, ds: f32, dt: f32) -> PlanarMapping2D {
        PlanarMapping2D { vs, vt, ds, dt }
    }
}

impl TextureMapping2D for PlanarMapping2D {
    fn map(&self, si: &SurfaceInteraction) -> (Point2f, Vector2f, Vector2f) {
        let vec = Vector3f::from(si.hit.p);
        (
            Point2f::new(self.ds + vec.dot(&self.vs), self.dt + vec.dot(&self.vt)),
            Vector2f::new(si.dpdx.dot(&self.vs), si.dpdx.dot(&self.vt)),
            Vector2f::new(si.dpdy.dot(&self.vs), si.dpdy.dot(&self.vt)),
        )
    }
}

pub trait TextureMapping3D: Debug + Send + Sync {
    fn map(&self, si: &SurfaceInteraction) -> (Point3f, Vector3f, Vector3f);
}

#[derive(Debug, Default)]
pub struct IdentityMapping3D {
    world_to_texture: Transform,
}

impl IdentityMapping3D {
    pub fn new(tex2world: Transform) -> IdentityMapping3D {
        IdentityMapping3D {
            world_to_texture: tex2world,
        }
    }
}

impl TextureMapping3D for IdentityMapping3D {
    fn map(&self, si: &SurfaceInteraction) -> (Point3f, Vector3f, Vector3f) {
        let dpdx = &self.world_to_texture * &si.dpdx;
        let dpdy = &self.world_to_texture * &si.dpdy;
        let p = &self.world_to_texture * &si.hit.p;

        (p, dpdx, dpdy)
    }
}
