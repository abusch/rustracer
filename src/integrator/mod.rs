use scene::Scene;
use ray::Ray;
use colour::Colourf;

mod whitted;

pub use self::whitted::Whitted;

pub trait Integrator {
    fn illumination(&self, scene: &Scene, ray: &mut Ray) -> Colourf;
}
