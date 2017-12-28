use std::f32;

use bounds::Bounds2i;
use spectrum::Spectrum;
use light_arena::Allocator;
use integrator::SamplerIntegrator;
use ray::Ray;
use sampler::Sampler;
use sampling::uniform_sample_sphere;
use scene::Scene;

pub struct AmbientOcclusion {
    pixel_bounds: Bounds2i,
    n_samples: usize,
}

impl AmbientOcclusion {
    pub fn new(n_samples: usize) -> AmbientOcclusion {
        AmbientOcclusion {
            n_samples: n_samples,
            pixel_bounds: Bounds2i::new(),
        }
    }
}

impl SamplerIntegrator for AmbientOcclusion {
    fn pixel_bounds(&self) -> &Bounds2i {
        &self.pixel_bounds
    }

    fn li(&self,
          scene: &Scene,
          ray: &mut Ray,
          sampler: &mut Box<Sampler>,
          _arena: &Allocator,
          _depth: u32)
          -> Spectrum {
        let mut n_clear: usize = 0;

        if let Some(intersection) = scene.intersect(ray) {
            let n = intersection.hit.n;
            for _ in 0..self.n_samples {
                let s = sampler.get_2d();
                let mut w = uniform_sample_sphere(&s);
                if w.dotn(&n) < 0.0 {
                    w = -w;
                }
                let ao_ray = intersection.spawn_ray(&w);
                if !scene.intersect_p(&ao_ray) {
                    n_clear += 1;
                }
            }
        }

        Spectrum::grey((n_clear as f32) / (self.n_samples as f32))
    }
}
