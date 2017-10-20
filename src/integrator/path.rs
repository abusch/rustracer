use light_arena::Allocator;

use bsdf::BxDFType;
use integrator::{uniform_sample_one_light, SamplerIntegrator};
use lightdistrib::{LightDistribution, UniformLightDistribution};
use material::TransportMode;
use ray::Ray;
use sampler::Sampler;
use scene::Scene;
use spectrum::Spectrum;

pub struct PathIntegrator {
    max_ray_depth: u8,
    rr_threshold: f32,
    light_sampling_strategy: String,
    light_distribution: Option<Box<LightDistribution + Send + Sync>>,
}

impl PathIntegrator {
    pub fn new() -> PathIntegrator {
        PathIntegrator {
            max_ray_depth: 5,
            rr_threshold: 1.0,
            light_sampling_strategy: "spatial".to_string(),
            light_distribution: None,
        }
    }
}

impl SamplerIntegrator for PathIntegrator {
    fn preprocess(&mut self, scene: &Scene, _sampler: &mut Box<Sampler + Send + Sync>) {
        // TODO create correct distribution based on strategy
        self.light_distribution = Some(Box::new(UniformLightDistribution::new(scene)));
    }

    fn li(
        &self,
        scene: &Scene,
        r: &mut Ray,
        sampler: &mut Box<Sampler + Send + Sync>,
        arena: &Allocator,
        _depth: u32,
    ) -> Spectrum {
        let mut l = Spectrum::black();
        let mut beta = Spectrum::white();
        let mut specular_bounce = false;
        let mut ray = *r;
        let mut bounces = 0;
        // Added after book publication: etaScale tracks the accumulated effect
        // of radiance scaling due to rays passing through refractive
        // boundaries (see the derivation on p. 527 of the third edition). We
        // track this value in order to remove it from beta when we apply
        // Russian roulette; this is worthwhile, since it lets us sometimes
        // avoid terminating refracted rays that are about to be refracted back
        // out of a medium and thus have their beta value increased.
        let mut eta_scale = 1.0;
        loop {
            // Find next path vertex and accumulate contribution
            debug!(
                "Path tracer bounce {}, current L={:?}, beta={:?}",
                bounces,
                l,
                beta
            );
            let mut found_intersection = scene.intersect(&mut ray);
            if bounces == 0 || specular_bounce {
                if let Some(ref isect) = found_intersection {
                    l += beta * isect.le(&(-ray.d));
                } else {
                    l = scene
                        .lights
                        .iter()
                        .fold(Spectrum::black(), |c, l| c + beta * l.le(&ray));
                }
            }

            // Terminate path if ray escaped or `max_depth` was reached
            if found_intersection.is_none() || bounces >= self.max_ray_depth {
                break;
            }

            // Compute scattering functions and skip over medium boundaries
            let isect = found_intersection.as_mut().unwrap();
            isect.compute_scattering_functions(&ray, TransportMode::RADIANCE, true, arena);
            if isect.bsdf.is_none() {
                // If there's no bsdf, it means we've hit the interface between two
                // different mediums. We simply continue along the same direction.
                ray = isect.spawn_ray(&ray.d);
                bounces -= 1;
                continue;
            }
            let bsdf = isect.bsdf.clone().unwrap();
            let distrib = self.light_distribution.as_ref().unwrap().lookup(&isect.p);

            // Sample illumination from lights to find path contribution.
            if bsdf.num_components(BxDFType::all() & !BxDFType::BSDF_SPECULAR) > 0 {
                let ld = beta * uniform_sample_one_light(isect, scene, sampler, distrib);
                assert!(ld.y() >= 0.0);
                l += ld;
            }

            // Sample BSDF to get new path direction
            let wo = -ray.d;
            let (f, wi, pdf, flags) = bsdf.sample_f(&wo, &sampler.get_2d(), BxDFType::all());
            if f.is_black() || pdf == 0.0 {
                break;
            }
            debug!("Update beta. beta={}, f={}, pdf={}", beta, f, pdf);
            beta = beta * f * wi.dot(&isect.shading.n).abs() / pdf;
            assert!(beta.y() >= 0.0);
            assert!(!beta.y().is_infinite());
            specular_bounce = flags.contains(BxDFType::BSDF_SPECULAR);
            if flags.contains(BxDFType::BSDF_SPECULAR)
                && flags.contains(BxDFType::BSDF_TRANSMISSION)
            {
                let eta = bsdf.eta;
                // Update the term that tracks radiance scaling for refraction
                // depending on whether the ray is entering or leaving the
                // medium.
                eta_scale *= if wo.dot(&isect.n) > 0.0 {
                    eta * eta
                } else {
                    1.0 / (eta * eta)
                };
            }

            ray = isect.spawn_ray(&wi);
            // Account for subsurface scattering, if applicable TODO

            // Possibly terminate the path with Russian roulette.
            // Factor out radiance scaling due to refraction in rr_beta.
            let rr_beta = beta * eta_scale;
            if rr_beta.max_component_value() < self.rr_threshold && bounces > 3 {
                let q = (1. - -rr_beta.max_component_value()).max(0.5);
                if sampler.get_1d() < q {
                    break;
                }
                beta = beta / (1.0 - q);
                assert!(!beta.y().is_infinite());
            }
            bounces += 1;
        }

        l
    }
}
