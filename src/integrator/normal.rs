use na::Dot;

use spectrum::Spectrum;
use integrator::SamplerIntegrator;
use ray::Ray;
use sampler::Sampler;
use scene::Scene;

pub struct Normal {}

impl SamplerIntegrator for Normal {
    fn li(&self, scene: &Scene, ray: &mut Ray, _: &mut Sampler, _: u32) -> Spectrum {
        if let Some(intersection) = scene.intersect(ray) {
            let n = intersection.n;
            Spectrum::grey(ray.d.dot(&n).abs())
        } else {
            Spectrum::black()
        }

    }
}
