use scene::Scene;
use ray::Ray;
use colour::Colourf;

mod whitted;
mod ao;

pub use self::whitted::Whitted;
pub use self::ao::AmbientOcclusion;

pub trait Integrator {
    fn illumination(&self, scene: &Scene, ray: &mut Ray) -> Colourf;
}
