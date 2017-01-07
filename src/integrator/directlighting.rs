use integrator::{SamplerIntegrator, uniform_sample_one_light};
use material::TransportMode;
use ray::Ray;
use sampler::Sampler;
use scene::Scene;
use spectrum::Spectrum;

pub enum LightStrategy {
    UniformSampleAll,
    UniformSampleOne,
}

pub struct DirectLightingIntegrator {
    pub light_strategy: LightStrategy,
    pub max_ray_depth: u8,
}

impl DirectLightingIntegrator {
    pub fn new(n: u8) -> DirectLightingIntegrator {
        DirectLightingIntegrator {
            max_ray_depth: n,
            light_strategy: LightStrategy::UniformSampleOne,
        }
    }
}

impl SamplerIntegrator for DirectLightingIntegrator {
    fn li(&self, scene: &Scene, ray: &mut Ray, sampler: &mut Sampler, depth: u32) -> Spectrum {
        let mut colour = Spectrum::black();

        match scene.intersect(ray) {
            Some(mut isect) => {
                let n = isect.shading.n;
                let wo = isect.wo;

                // Compute scattering functions for surface interaction
                isect.compute_scattering_functions(ray, TransportMode::RADIANCE, false);

                if isect.bsdf.is_none() {
                    let mut r = isect.spawn_ray(&ray.d);
                    return self.li(scene, &mut r, sampler, depth);
                }
                let bsdf = isect.bsdf.clone().unwrap();

                // Compute emitted light if ray hit an area light source
                colour += isect.le(&wo);
                if !scene.lights.is_empty() {
                    // Compute direct lighting for DirectLightingIntegrator
                    colour += match self.light_strategy {
                        LightStrategy::UniformSampleAll => unimplemented!(),
                        LightStrategy::UniformSampleOne => {
                            uniform_sample_one_light(&isect, scene, sampler)
                        }
                    }
                }

                if depth + 1 < self.max_ray_depth as u32 {
                    colour += self.specular_reflection(ray, &isect, scene, &bsdf, sampler, depth);
                    colour += self.specular_transmission(ray, &isect, scene, &bsdf, sampler, depth);
                }
            }
            None => {
                colour = scene.lights.iter().fold(Spectrum::black(), |c, l| c + l.le(ray));
            }
        }


        colour
    }
}
