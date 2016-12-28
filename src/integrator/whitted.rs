use na::Dot;

use bsdf;
use integrator::SamplerIntegrator;
use material::TransportMode;
use ray::Ray;
use sampler::Sampler;
use scene::Scene;
use spectrum::Spectrum;

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

                // Add contribution of each light source
                for light in &scene.lights {
                    let (li, wi, pdf, visibility_tester) =
                        light.sample_li(&isect, &wo, &sampler.get_2d());
                    if li.is_black() || pdf == 0.0 {
                        continue;
                    }

                    let f = bsdf.f(&wo, &wi, bsdf::BxDFType::all());
                    if !f.is_black() && visibility_tester.unoccluded(scene) {
                        colour += f * li * wi.dot(&n).abs() / pdf;
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
