use std::sync::Arc;

use light_arena::Allocator;

use bounds::Bounds2i;
use bsdf::BxDFType;
use camera::Camera;
use integrator::{uniform_sample_one_light, SamplerIntegrator};
use lightdistrib::{LightDistribution, UniformLightDistribution, SpatialLightDistribution};
use material::TransportMode;
use paramset::ParamSet;
use ray::Ray;
use sampler::Sampler;
use scene::Scene;
use spectrum::Spectrum;

stat_percent!("Integrator/Zero-radiance paths", zero_radiance_paths);
stat_int_distribution!("Integrator/Path length", path_length);
pub fn init_stats() {
    zero_radiance_paths::init();
    path_length::init();
}

pub struct PathIntegrator {
    pixel_bounds: Bounds2i,
    max_ray_depth: u8,
    rr_threshold: f32,
    light_sampling_strategy: String,
    light_distribution: Option<Box<LightDistribution>>,
}

impl PathIntegrator {
    pub fn new(pixel_bounds: Bounds2i,
               max_ray_depth: i32,
               rr_threshold: f32,
               light_sampling_strategy: String)
               -> PathIntegrator {
        PathIntegrator {
            pixel_bounds,
            max_ray_depth: max_ray_depth as u8,
            rr_threshold,
            light_sampling_strategy,
            light_distribution: None,
        }
    }

    pub fn create(params: &mut ParamSet,
                  camera: &Camera)
                  -> Box<SamplerIntegrator> {
        let max_depth = params.find_one_int("maxdepth", 5);
        let rr_threshold = params.find_one_float("rrthreshold", 1.0);
        let light_strategy = params.find_one_string("lightsamplestrategy", "spatial".into());
        let pb = params.find_int("pixelbounds");
        let mut pixel_bounds = camera.get_film().get_sample_bounds();
        if let Some(pb) = pb {
            if pb.len() != 4 {
                error!("Expected 4 values for \"pixelbounds\" parameter. Got {}.",
                       pb.len());
            } else {
                pixel_bounds =
                    Bounds2i::intersect(&pixel_bounds,
                                        &Bounds2i::from_elements(pb[0], pb[2], pb[1], pb[3]));
                if pixel_bounds.area() == 0 {
                    error!("Degenerate \"pixelbounds\" specified. Ignoring.");
                }
            }
        }

        Box::new(PathIntegrator::new(pixel_bounds, max_depth, rr_threshold, light_strategy))
    }
}

impl SamplerIntegrator for PathIntegrator {
    fn pixel_bounds(&self) -> &Bounds2i {
        &self.pixel_bounds
    }

    fn preprocess(&mut self, scene: Arc<Scene>, _sampler: &mut Box<Sampler>) {
        // TODO create correct distribution based on strategy
        self.light_distribution = if self.light_sampling_strategy == "uniform" ||
                                     scene.lights.len() == 1 {
            Some(Box::new(UniformLightDistribution::new(&scene)))
        } else {
            Some(Box::new(SpatialLightDistribution::new(scene, 64)))
        }
    }

    fn li(&self,
          scene: &Scene,
          r: &mut Ray,
          sampler: &mut Box<Sampler>,
          arena: &Allocator,
          _depth: u32)
          -> Spectrum {
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
            debug!("Path tracer bounce {}, current L={:?}, beta={:?}",
                   bounces,
                   l,
                   beta);
            // Intersect _ray_ with scene and store intersection in _isect_
            let mut found_intersection = scene.intersect(&mut ray);

            // Possibly add emitted light at intersection
            if bounces == 0 || specular_bounce {
                // Add emitted light at path vertex or from the environment
                if let Some(ref isect) = found_intersection {
                    l += beta * isect.le(&(-ray.d));
                } else {
                    for light in &scene.infinite_lights {
                        l += beta * light.le(&ray);
                    }
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
            let distrib = self.light_distribution
                .as_ref()
                .unwrap()
                .lookup(&isect.hit.p);

            // Sample illumination from lights to find path contribution.
            if bsdf.num_components(BxDFType::all() & !BxDFType::BSDF_SPECULAR) > 0 {
                zero_radiance_paths::inc_total();
                let ld = beta * uniform_sample_one_light(isect, scene, sampler, distrib);
                if ld.is_black() {
                    zero_radiance_paths::inc();
                }
                assert!(ld.y() >= 0.0);
                l += ld;
            }

            // Sample BSDF to get new path direction
            let wo = -ray.d;
            let (f, wi, pdf, flags) = bsdf.sample_f(&wo, &sampler.get_2d(), BxDFType::all());
            if f.is_black() || pdf <= 0.0 {
                break;
            }
            debug!("Update beta. beta={}, f={}, pdf={}", beta, f, pdf);
            beta = beta * f * wi.dotn(&isect.shading.n).abs() / pdf;
            assert!(beta.y() >= 0.0);
            // assert!(!beta.y().is_infinite());
            specular_bounce = flags.contains(BxDFType::BSDF_SPECULAR);
            if flags.contains(BxDFType::BSDF_SPECULAR) &&
               flags.contains(BxDFType::BSDF_TRANSMISSION) {
                let eta = bsdf.eta;
                // Update the term that tracks radiance scaling for refraction
                // depending on whether the ray is entering or leaving the
                // medium.
                eta_scale *= if wo.dotn(&isect.hit.n) > 0.0 {
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
                let q = (1.0 - rr_beta.max_component_value()).max(0.05);
                if sampler.get_1d() < q {
                    break;
                }
                beta = beta / (1.0 - q);
                assert!(!beta.y().is_infinite());
            }
            bounces += 1;
        }

        path_length::report_value(bounces as u64);
        l
    }
}
