use std::f32::consts::*;

use bsdf;
use spectrum::Spectrum;
use integrator::SamplerIntegrator;
use ray::Ray;
use sampling::Sampler;
use scene::Scene;
use material::TransportMode;
use na::Dot;

pub struct Whitted {
    pub max_ray_depth: u8,
}

impl Whitted {
    pub fn new(n: u8) -> Whitted {
        Whitted { max_ray_depth: n }
    }
}

impl SamplerIntegrator for Whitted {
    fn li(&self, scene: &Scene, ray: &mut Ray, sampler: &mut Sampler, depth: u32) -> Spectrum {
        let mut colour = Spectrum::black();

        match scene.intersect2(ray) {
            Some(mut isect) => {
                let n = isect.shading.n;
                let p = isect.p;
                let wo = isect.wo;

                // Compute scattering functions for surface interaction
                isect.compute_scattering_functions(ray, TransportMode::RADIANCE, false);

                // Compute emitted light if ray hit an area light source
                colour += isect.le(wo);

                // Add contribution of each light source
                // let bsdf = isect.material.bsdf(&isect);


                if isect.bsdf.is_none() {
                    let mut r = isect.spawn_ray(&ray.d);
                    return self.li(scene, &mut r, sampler, depth);
                }
                let bsdf = isect.bsdf.clone().unwrap();

                for light in &scene.lights {
                    let (li, wi, pdf, visibilityTester) = light.sample_li(&isect, &wo, (0.0, 0.0));
                    if li.is_black() || pdf == 0.0 {
                        continue;
                    }

                    let f = bsdf.f(&wi, &wo, bsdf::BxDFType::all());
                    if !f.is_black() && visibilityTester.unoccluded(scene) {
                        colour += f * li * wi.dot(&n).abs() / pdf;
                    }
                }

                if depth + 1 < self.max_ray_depth as u32 {
                    colour += self.specular_reflection(ray, &isect, scene, &bsdf, sampler, depth);
                    colour += self.specular_transmission(ray, &isect, scene, &bsdf, sampler, depth);
                }
            }
            None => {
                // colour = scene.atmosphere.compute_incident_light(ray);
                colour = Spectrum::black();
            }
        }


        colour
    }
}
