use light_arena::Allocator;

use bounds::Bounds2i;
use spectrum::Spectrum;
use integrator::SamplerIntegrator;
use ray::Ray;
use sampler::Sampler;
use scene::Scene;

#[derive(Default)]
pub struct Normal {
    pixel_bounds: Bounds2i,
}

impl SamplerIntegrator for Normal {
    fn pixel_bounds(&self) -> &Bounds2i {
        &self.pixel_bounds
    }

    fn li(&self,
          scene: &Scene,
          ray: &mut Ray,
          _sampler: &mut Box<Sampler>,
          _arena: &Allocator,
          _depth: u32)
          -> Spectrum {
        if let Some(intersection) = scene.intersect(ray) {
            let n = intersection.hit.n;
            Spectrum::grey(ray.d.dotn(&n).abs())
        } else {
            Spectrum::black()
        }
    }
}
