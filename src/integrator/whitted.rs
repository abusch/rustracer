use std::f32::consts::*;

use Vector;
use bsdf;
use colour::Colourf;
use integrator::SamplerIntegrator;
use ray::Ray;
use sampling::Sampler;
use scene::Scene;
use na::{Norm, Dot, zero};
use na;

pub struct Whitted {
    pub max_ray_depth: u8,
}

impl Whitted {
    pub fn new(n: u8) -> Whitted {
        Whitted { max_ray_depth: n }
    }
}

impl SamplerIntegrator for Whitted {
    fn li(&self, scene: &Scene, ray: &mut Ray, sampler: &mut Sampler, depth: u32) -> Colourf {
        let mut colour = Colourf::black();

        match scene.intersect(ray) {
            Some(intersection) => {
                let n = intersection.dg.nhit;
                let p = intersection.dg.phit;
                let wo = intersection.wo;

                // Compute scattering functions for surface interaction
                // TODO

                // Compute emitted light if ray hit an area light source
                colour += intersection.le(wo);

                // Add contribution of each light source
                let bsdf = intersection.material.bsdf(&intersection);
                for light in &scene.lights {
                    let (li, wi, pdf) = light.sample_li(&intersection, &wo, (0.0, 0.0));
                    if li.is_black() || pdf == 0.0 {
                        continue;
                    }

                    // TODO VisibilityTester
                    let mut shadow_ray = ray.spawn(p, -wi);
                    // shadow_ray.t_max = shading_info.light_distance;
                    let f = bsdf.f(&wi, &wo);
                    if !f.is_black() && !scene.intersect_p(&mut shadow_ray) {
                        // TODO Why do I still have to divide by PI?
                        colour += f * li * wi.dot(&n).abs() * FRAC_1_PI / pdf;
                    }
                }

                if depth + 1 < self.max_ray_depth as u32 {
                    colour +=
                        self.specular_reflection(ray, &intersection, scene, &bsdf, sampler, depth);
                    colour +=
                        self.specular_transmission(ray,
                                                   &intersection,
                                                   scene,
                                                   &bsdf,
                                                   sampler,
                                                   depth);
                }
            }
            None => {
                colour = scene.atmosphere.compute_incident_light(ray);
            }
        }

        colour
    }
}
