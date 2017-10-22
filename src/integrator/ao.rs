use std::f32;

use spectrum::Spectrum;
use light_arena::Allocator;
use integrator::SamplerIntegrator;
use ray::Ray;
use sampler::Sampler;
use sampling::uniform_sample_sphere;
use scene::Scene;

pub struct AmbientOcclusion {
    n_samples: usize,
}

impl AmbientOcclusion {
    pub fn new(n_samples: usize) -> AmbientOcclusion {
        AmbientOcclusion {
            n_samples: n_samples,
        }
    }
}

impl SamplerIntegrator for AmbientOcclusion {
    fn li(
        &self,
        scene: &Scene,
        ray: &mut Ray,
        sampler: &mut Box<Sampler + Send + Sync>,
        _arena: &Allocator,
        _depth: u32,
    ) -> Spectrum {
        let mut n_clear: usize = 0;

        if let Some(intersection) = scene.intersect(ray) {
            let n = intersection.n;
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
