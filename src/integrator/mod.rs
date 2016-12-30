use std::sync::Arc;
use std::cmp;

use na::Dot;

use ::Point2f;
use bsdf;
use spectrum::Spectrum;
use interaction::SurfaceInteraction;
use light::{Light, is_delta_light};
use primitive::Primitive;
use ray::Ray;
use sampler::Sampler;
use sampling::power_heuristic;
use scene::Scene;

mod whitted;
mod directlighting;
mod ao;
mod normal;

pub use self::whitted::Whitted;
pub use self::directlighting::DirectLightingIntegrator;
pub use self::ao::AmbientOcclusion;
pub use self::normal::Normal;

pub trait SamplerIntegrator {
    fn li(&self, scene: &Scene, ray: &mut Ray, sampler: &mut Sampler, depth: u32) -> Spectrum;

    fn specular_reflection(&self,
                           ray: &mut Ray,
                           isect: &SurfaceInteraction,
                           scene: &Scene,
                           bsdf: &bsdf::BSDF,
                           sampler: &mut Sampler,
                           depth: u32)
                           -> Spectrum {
        let flags = bsdf::BSDF_REFLECTION | bsdf::BSDF_SPECULAR;
        let (f, wi, pdf, bsdf_type) = bsdf.sample_f(&isect.wo, &sampler.get_2d(), flags);
        let ns = isect.shading.n;
        if !f.is_black() && pdf != 0.0 && wi.dot(&ns) != 0.0 {
            let mut r = ray.spawn(isect.p, wi);
            let refl = self.li(scene, &mut r, sampler, depth + 1);
            f * refl * wi.dot(&ns).abs() / pdf
        } else {
            Spectrum::black()
        }
    }

    fn specular_transmission(&self,
                             ray: &mut Ray,
                             isect: &SurfaceInteraction,
                             scene: &Scene,
                             bsdf: &bsdf::BSDF,
                             sampler: &mut Sampler,
                             depth: u32)
                             -> Spectrum {
        let flags = bsdf::BSDF_TRANSMISSION | bsdf::BSDF_SPECULAR;
        // TODO use sampler.get_2d()
        let (f, wi, pdf, bsdf_type) = bsdf.sample_f(&isect.wo, &sampler.get_2d(), flags);
        let ns = isect.shading.n;
        if !f.is_black() && pdf != 0.0 && wi.dot(&ns) != 0.0 {
            let mut r = ray.spawn(isect.p, wi);
            let refr = self.li(scene, &mut r, sampler, depth + 1);
            f * refr * wi.dot(&ns).abs() / pdf
        } else {
            Spectrum::black()
        }
    }
}

pub fn uniform_sample_one_light(it: &SurfaceInteraction,
                                scene: &Scene,
                                sampler: &mut Sampler)
                                -> Spectrum {
    let n_lights = scene.lights.len();
    if n_lights == 0 {
        Spectrum::black()
    } else {
        // Randomly chose a light to sample
        let num_light = cmp::max(n_lights - 1, (sampler.get_1d() * n_lights as f32) as usize);
        let light = &scene.lights[num_light];
        let u_light = sampler.get_2d();
        let u_scattering = sampler.get_2d();
        estimate_direct(it, &u_scattering, light, &u_light, scene, sampler) * n_lights as f32
    }
}

pub fn estimate_direct(it: &SurfaceInteraction,
                       u_scattering: &Point2f,
                       light: &Arc<Light + Send + Sync>,
                       u_light: &Point2f,
                       scene: &Scene,
                       sampler: &mut Sampler)
                       -> Spectrum {
    // let light = light.as_ref();
    let bsdf_flags = bsdf::BxDFType::all();
    let mut ld = Spectrum::black();
    // Sample light with multiple importance sampling
    let bsdf = it.bsdf.as_ref().expect("There should be a BSDF set at this point!");
    let (mut li, wi, light_pdf, vis) = light.sample_li(it, &u_light);
    if light_pdf > 0.0 && !li.is_black() {
        // Compute BSDF for light sample
        let f = bsdf.f(&it.wo, &wi, bsdf_flags) * wi.dot(&it.shading.n).abs();
        let scattering_pdf = bsdf.pdf(&it.wo, &wi, bsdf_flags);
        if !f.is_black() {
            if !vis.unoccluded(scene) {
                li = Spectrum::black();
            }
            // Add light's contribution to reflected radiance
            if !li.is_black() {
                if is_delta_light(light.flags()) {
                    ld += f * li / light_pdf;
                } else {
                    let weight = power_heuristic(1, light_pdf, 1, scattering_pdf);
                    ld += f * li / weight * light_pdf;
                }
            }
        }
        // TODO compute phase function for medium interaction when supported
    }
    // Sample BSDF with multiple importance sampling
    if !is_delta_light(light.flags()) {
        let (mut f, wi, scattering_pdf, sampled_type) =
            bsdf.sample_f(&it.wo, u_scattering, bsdf_flags);
        f = f * wi.dot(&it.shading.n).abs();
        let sampled_specular = sampled_type.contains(bsdf::BSDF_SPECULAR);
        // TODO compute medium interaction when supported
        if !f.is_black() && scattering_pdf > 0.0 {
            // Account for light contribution along sampled direction wi
            let mut weight = 1.0;
            if !sampled_specular {
                let light_pdf = light.pdf_li(it, &wi);
                if light_pdf == 0.0 {
                    return ld;
                }
                weight = power_heuristic(1, scattering_pdf, 1, light_pdf);
            }

            // Find intersection and compute transmittance
            let mut ray = it.spawn_ray(&wi);
            let li = match scene.intersect(&mut ray) {
                Some(light_isect) => {
                    // Add light contribution from material sampling
                    if let Some(area_light) = light_isect.primitive.and_then(|p| p.area_light()) {
                        if area_light.id() == light.id() {
                            light_isect.le(&(-wi))
                        } else {
                            Spectrum::black()
                        }
                    } else {
                        Spectrum::black()
                    }
                }
                None => light.le(&ray),
            };
            if !li.is_black() {
                ld += f * li * weight / scattering_pdf;
            }
        }
    }

    ld
}
