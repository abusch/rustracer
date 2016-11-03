use colour::Colourf;
use ray::Ray;
use sampling::Sampler;
use scene::Scene;

mod whitted;
mod ao;
mod normal;

pub use self::whitted::Whitted;
pub use self::ao::AmbientOcclusion;
pub use self::normal::Normal;

pub trait SamplerIntegrator {
    fn li(&self, scene: &Scene, ray: &mut Ray, sampler: &mut Sampler) -> Colourf;
}
