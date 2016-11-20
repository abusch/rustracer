use std::f32;
use std::iter;

use Vector;
use spectrum::Spectrum;
use integrator::SamplerIntegrator;
use ray::Ray;
use sampling::Sampler;
use scene::Scene;
use na::Dot;

pub struct AmbientOcclusion {
    n_samples: usize,
}

impl AmbientOcclusion {
    pub fn new(n_samples: usize) -> AmbientOcclusion {
        AmbientOcclusion { n_samples: n_samples }
    }
}

impl SamplerIntegrator for AmbientOcclusion {
    fn li(&self, scene: &Scene, ray: &mut Ray, sampler: &mut Sampler, _: u32) -> Spectrum {
        let mut n_clear: usize = 0;

        if let Some(intersection) = scene.intersect(ray) {
            let n = intersection.dg.nhit;
            let p = intersection.dg.phit;
            let mut samples = iter::repeat((0.0, 0.0)).take(self.n_samples).collect();

            sampler.get_samples(0.0, 0.0, &mut samples);
            for s in &samples {
                let mut w = uniform_sample_sphere(s.0, s.1);
                if w.dot(&n) < 0.0 {
                    w = -w;
                }
                let mut ao_ray = ray.spawn(p, w);
                if scene.intersect(&mut ao_ray).is_none() {
                    n_clear += 1;
                }

            }
        }

        Spectrum::grey((n_clear as f32) / (self.n_samples as f32))
    }
}
