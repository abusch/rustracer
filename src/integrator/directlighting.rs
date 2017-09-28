use integrator::{uniform_sample_one_light, SamplerIntegrator};
use material::TransportMode;
use paramset::ParamSet;
use ray::Ray;
use sampler::Sampler;
use scene::Scene;
use spectrum::Spectrum;

/// Strategy to use for sampling lights
pub enum LightStrategy {
    /// For each pixel sample, sample every light in the scene
    UniformSampleAll,
    /// For each pixel sample, only sample one light from the scene, chosen at random
    UniformSampleOne,
}

/// Integrator that only takes into account direct lighting i.e no global illumination. It is very
/// similar to the Whitted integrator but has a better light sampling strategy.
pub struct DirectLightingIntegrator {
    /// The strategy to use to sample lights
    pub light_strategy: LightStrategy,
    /// Maximum number of times a ray can bounce before terminating
    pub max_ray_depth: u8,
}

impl DirectLightingIntegrator {
    pub fn new(n: u8, strategy: LightStrategy) -> DirectLightingIntegrator {
        DirectLightingIntegrator {
            max_ray_depth: n,
            light_strategy: strategy,
        }
    }

    pub fn create(ps: &mut ParamSet) -> Box<SamplerIntegrator + Send + Sync> {
        let max_depth = ps.find_one_int("maxdepth", 5);
        let st = ps.find_one_string("strategy", "all".into());
        let strategy = if st == "one" {
            LightStrategy::UniformSampleOne
        } else if st == "all" {
            LightStrategy::UniformSampleAll
        } else {
            warn!("Strategy \"{}\" for directlighting unknown. Using \"all\".");
            LightStrategy::UniformSampleAll
        };
        // TODO pixel_bounds
        Box::new(Self::new(max_depth as u8, strategy))
    }
}

impl SamplerIntegrator for DirectLightingIntegrator {
    fn li(
        &self,
        scene: &Scene,
        ray: &mut Ray,
        sampler: &mut Box<Sampler + Send + Sync>,
        depth: u32,
    ) -> Spectrum {
        let mut colour = Spectrum::black();

        match scene.intersect(ray) {
            Some(mut isect) => {
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
                            uniform_sample_one_light(&isect, scene, sampler, None)
                        }
                    }
                }

                if depth + 1 < self.max_ray_depth as u32 {
                    colour += self.specular_reflection(ray, &isect, scene, &bsdf, sampler, depth);
                    colour += self.specular_transmission(ray, &isect, scene, &bsdf, sampler, depth);
                }
            }
            None => {
                // If we didn't intersect anything, add the backgound radiance from every light
                colour = scene
                    .lights
                    .iter()
                    .fold(Spectrum::black(), |c, l| c + l.le(ray));
            }
        }

        colour
    }
}
