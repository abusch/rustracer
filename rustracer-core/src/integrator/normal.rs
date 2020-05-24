use light_arena::Allocator;

use crate::bounds::Bounds2i;
use crate::integrator::SamplerIntegrator;
use crate::ray::Ray;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::spectrum::Spectrum;

#[derive(Default)]
pub struct Normal {
    pixel_bounds: Bounds2i,
}

impl SamplerIntegrator for Normal {
    fn pixel_bounds(&self) -> &Bounds2i {
        &self.pixel_bounds
    }

    fn li(
        &self,
        scene: &Scene,
        ray: &mut Ray,
        _sampler: &mut dyn Sampler,
        _arena: &Allocator,
        _depth: u32,
    ) -> Spectrum {
        if let Some(intersection) = scene.intersect(ray) {
            let n = intersection.hit.n;
            Spectrum::grey(ray.d.dotn(&n).abs())
        } else {
            Spectrum::black()
        }
    }
}
