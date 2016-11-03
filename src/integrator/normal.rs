use na::Dot;

use colour::Colourf;
use integrator::SamplerIntegrator;
use ray::Ray;
use sampling::Sampler;
use scene::Scene;

pub struct Normal {
}

impl SamplerIntegrator for Normal {
    fn li(&self, scene: &Scene, ray: &mut Ray, _: &mut Sampler, _: u32) -> Colourf {
        if let Some(intersection) = scene.intersect(ray) {
            let n = intersection.dg.nhit;
            Colourf::grey(ray.dir.dot(&n).abs())
        } else {
            Colourf::black()
        }

    }
}
