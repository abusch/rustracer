use std::f32::consts::*;

use bsdf;
use colour::Colourf;
use integrator::SamplerIntegrator;
use ray::Ray;
use sampling::Sampler;
use scene::Scene;
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
    fn li(&self, scene: &Scene, ray: &mut Ray, sampler: &mut Sampler, depth: u32) -> Colourf {
        let mut colour = Colourf::black();

        match scene.intersect2(ray) {
            Some(isect) => {
                let n = isect.shading.n;
                let p = isect.p;
                let wo = isect.wo;

                // Compute scattering functions for surface interaction

                // Compute emitted light if ray hit an area light source
                colour += isect.le(wo);

                // Add contribution of each light source
                // let bsdf = isect.material.bsdf(&isect);
                let bxdfs: Vec<Box<bsdf::BxDF + Send + Sync>> = vec![Box::new(bsdf::SpecularReflection::new(Colourf::rgb(1.0, 0.0, 0.0),
                                                         Box::new(bsdf::FresnelConductor::new(
                                                                 Colourf::white(),
                                                                 Colourf::rgb(0.155265, 0.116723, 0.138381),
                                                                 Colourf::rgb(4.82835, 3.12225, 2.14696),
                                                                 ))))];
                let bsdf = bsdf::BSDF::new(&isect, 1.5, &bxdfs[..]);
                for light in &scene.lights {
                    let (li, wi, pdf) = light.sample_li(&isect, &wo, (0.0, 0.0));
                    if li.is_black() || pdf == 0.0 {
                        continue;
                    }

                    // TODO VisibilityTester
                    let mut shadow_ray = ray.spawn(p, -wi);
                    // shadow_ray.t_max = shading_info.light_distance;
                    let f = bsdf.f(&wi, &wo, bsdf::BxDFType::all());
                    if !f.is_black() && !scene.intersect_p(&mut shadow_ray) {
                        // TODO Why do I still have to divide by PI?
                        colour += f * li * wi.dot(&n).abs() * FRAC_1_PI / pdf;
                    }
                }

                if depth + 1 < self.max_ray_depth as u32 {
                    colour += self.specular_reflection(ray, &isect, scene, &bsdf, sampler, depth);
                    colour += self.specular_transmission(ray, &isect, scene, &bsdf, sampler, depth);
                }
            }
            None => {
                colour = scene.atmosphere.compute_incident_light(ray);
            }
        }

        colour
    }
}
