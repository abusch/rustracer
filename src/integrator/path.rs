use na::Dot;

use bsdf::{self, BxDFType};
use integrator::{SamplerIntegrator, uniform_sample_one_light};
use lightdistrib::{LightDistribution, UniformLightDistribution};
use material::TransportMode;
use ray::Ray;
use sampler::Sampler;
use scene::Scene;
use spectrum::Spectrum;

pub struct PathIntegrator {
    max_ray_depth: u8,
    rr_threshold: f32,
    light_sampling_strategy: Box<LightDistribution + Send + Sync>,
}

impl PathIntegrator {
    pub fn new(scene: &Scene) -> PathIntegrator {
        PathIntegrator {
            max_ray_depth: 5,
            rr_threshold: 1.0,
            light_sampling_strategy: Box::new(UniformLightDistribution::new(scene)),
        }
    }
}

impl SamplerIntegrator for PathIntegrator {
    fn li(&self, scene: &Scene, r: &mut Ray, sampler: &mut Sampler, _depth: u32) -> Spectrum {
        let mut l = Spectrum::black();
        let mut beta = Spectrum::white();
        let mut specular_bounce = false;
        let mut eta_scale = 1.0;
        let mut ray = *r;

        let mut bounces = 0;
        loop {
            debug!("Path tracer bounce {}, current L={:?}, beta={:?}",
                   bounces,
                   l,
                   beta);
            match scene.intersect(&mut ray) {
                None => {
                    if bounces == 0 || specular_bounce {
                        l = scene.lights
                            .iter()
                            .fold(Spectrum::black(), |c, l| c + beta * l.le(&ray));
                        // info!("Added infinite area lights -> L={:?}", l);
                    }
                    break;
                }
                Some(mut isect) => {
                    if bounces == 0 || specular_bounce {
                        l += beta * isect.le(&(-ray.d));
                    }
                    if bounces >= self.max_ray_depth {
                        break;
                    }

                    isect.compute_scattering_functions(&ray, TransportMode::RADIANCE, true);

                    if isect.bsdf.is_none() {
                        // If there's no bsdf, it means we've hit the interface between two
                        // different mediums. We simply continue along the same direction.
                        ray = isect.spawn_ray(&ray.d);
                        bounces -= 1;
                        continue;
                    }
                    let bsdf = isect.bsdf.clone().unwrap();
                    let distrib = self.light_sampling_strategy.lookup(&isect.p);

                    if bsdf.num_components(BxDFType::all() & !bsdf::BSDF_SPECULAR) > 0 {
                        let ld = beta * uniform_sample_one_light(&isect, scene, sampler, distrib);
                        assert!(ld.y() >= 0.0);
                        l += ld;
                    }

                    let wo = -ray.d;
                    let (f, wi, pdf, flags) =
                        bsdf.sample_f(&wo, &sampler.get_2d(), BxDFType::all());
                    if f.is_black() || pdf == 0.0 {
                        break;
                    }
                    beta = beta * f * wi.dot(&isect.shading.n).abs() / pdf;
                    assert!(beta.y() >= 0.0);
                    assert!(!beta.y().is_infinite());
                    specular_bounce = flags.contains(bsdf::BSDF_SPECULAR);
                    if flags.contains(bsdf::BSDF_SPECULAR) &&
                       flags.contains(bsdf::BSDF_TRANSMISSION) {
                        let eta = bsdf.eta;
                        eta_scale *= if wo.dot(&isect.n) > 0.0 {
                            eta * eta
                        } else {
                            1.0 / (eta * eta)
                        };
                    }

                    ray = isect.spawn_ray(&wi);

                    let rr_beta = beta * eta_scale;
                    if rr_beta.max_component_value() < self.rr_threshold && bounces > 3 {
                        let q = (1. - -rr_beta.max_component_value()).max(0.5);
                        if sampler.get_1d() < q {
                            break;
                        }
                        beta = beta / (1.0 - q);
                        assert!(!beta.y().is_infinite());
                    }
                }
            }

            bounces += 1;
        }

        l
    }
}
