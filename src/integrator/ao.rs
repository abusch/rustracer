use std::f32;
use std::iter;

use Vector;
use colour::Colourf;
use integrator::Integrator;
use ray::Ray;
use sampling::{Sampler, LowDiscrepancy};
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

fn uniform_sample_sphere(x: f32, y: f32) -> Vector {
    let z = 1.0 - 2.0 * x;
    let r = f32::sqrt(f32::max(0.0, 1.0 - z * z));
    let phi = 2.0 * f32::consts::PI * y;

    Vector::new(r * f32::cos(phi), r * f32::sin(phi), z)
}

impl Integrator for AmbientOcclusion {
    fn illumination(&self, scene: &Scene, ray: &mut Ray) -> Colourf {
        let mut n_clear: usize = 0;
        let sampler = LowDiscrepancy::new(self.n_samples);

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
                if let None = scene.intersect(&mut ao_ray) {
                    n_clear += 1;
                }

            }
        }

        Colourf::grey((n_clear as f32) / (self.n_samples as f32))
    }
}
