use light_arena::Allocator;
use integrator::{uniform_sample_all_light, uniform_sample_one_light, SamplerIntegrator};
use material::TransportMode;
use paramset::ParamSet;
use ray::Ray;
use sampler::Sampler;
use scene::Scene;
use spectrum::Spectrum;

/// Strategy to use for sampling lights
#[derive(PartialEq, Eq)]
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
    pub max_depth: u8,
    //
    n_light_samples: Vec<usize>,
}

impl DirectLightingIntegrator {
    pub fn new(n: u8, strategy: LightStrategy) -> DirectLightingIntegrator {
        DirectLightingIntegrator {
            max_depth: n,
            light_strategy: strategy,
            n_light_samples: Vec::new(),
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
            warn!(
                "Strategy \"{}\" for directlighting unknown. Using \"all\".",
                st
            );
            LightStrategy::UniformSampleAll
        };
        // TODO pixel_bounds
        Box::new(Self::new(max_depth as u8, strategy))
    }
}

impl SamplerIntegrator for DirectLightingIntegrator {
    fn preprocess(&mut self, scene: &Scene, sampler: &mut Box<Sampler + Send + Sync>) {
        info!("Preprocessing DirectLighting integrator");
        if self.light_strategy == LightStrategy::UniformSampleAll {
            // Compute number of samples to use for each light
            for light in &scene.lights {
                self.n_light_samples
                    .push(sampler.round_count(light.n_samples() as usize));
            }
            info!("n sample sizes: {:?}", self.n_light_samples);

            for _i in 0..self.max_depth {
                for j in 0..scene.lights.len() {
                    sampler.request_2d_array(self.n_light_samples[j]);
                    sampler.request_2d_array(self.n_light_samples[j]);
                }
            }
        }
    }

    fn li(
        &self,
        scene: &Scene,
        ray: &mut Ray,
        sampler: &mut Box<Sampler + Send + Sync>,
        depth: u32,
        arena: &Allocator,
    ) -> Spectrum {
        let mut colour = Spectrum::black();

        match scene.intersect(ray) {
            Some(mut isect) => {
                let wo = isect.wo;

                // Compute scattering functions for surface interaction
                isect.compute_scattering_functions(ray, TransportMode::RADIANCE, false, arena);

                if isect.bsdf.is_none() {
                    let mut r = isect.spawn_ray(&ray.d);
                    return self.li(scene, &mut r, sampler, depth, arena);
                }
                let bsdf = isect.bsdf.clone().unwrap();

                // Compute emitted light if ray hit an area light source
                colour += isect.le(&wo);
                if !scene.lights.is_empty() {
                    // Compute direct lighting for DirectLightingIntegrator
                    colour += match self.light_strategy {
                        LightStrategy::UniformSampleAll => {
                            uniform_sample_all_light(&isect, scene, sampler, &self.n_light_samples)
                        }
                        LightStrategy::UniformSampleOne => {
                            uniform_sample_one_light(&isect, scene, sampler, None)
                        }
                    }
                }

                if depth + 1 < self.max_depth as u32 {
                    colour +=
                        self.specular_reflection(ray, &isect, scene, &bsdf, sampler, depth, arena);
                    colour += self.specular_transmission(
                        ray,
                        &isect,
                        scene,
                        &bsdf,
                        sampler,
                        depth,
                        arena,
                    );
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
