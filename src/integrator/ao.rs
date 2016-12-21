use std::f32;
use std::iter;

use ::Point2f;
use spectrum::Spectrum;
use integrator::SamplerIntegrator;
use ray::Ray;
use sampler::Sampler;
use sampling::uniform_sample_sphere;
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
            let n = intersection.n;
            let mut samples: Vec<Point2f> =
                iter::repeat(Point2f::new(0.0, 0.0)).take(self.n_samples).collect();

            // TODO fixme
            // sampler.get_samples(0.0, 0.0, &mut samples);
            for s in &samples {
                let mut w = uniform_sample_sphere(&s);
                if w.dot(&n) < 0.0 {
                    w = -w;
                }
                let mut ao_ray = intersection.spawn_ray(&w);
                if !scene.intersect_p(&mut ao_ray) {
                    n_clear += 1;
                }

            }
        }

        Spectrum::grey((n_clear as f32) / (self.n_samples as f32))
    }
}
