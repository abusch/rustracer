use std::sync::Arc;
use std::cmp;

use {Point2f, Vector3f};
use bsdf::{self, BxDFType};
use spectrum::Spectrum;
use interaction::SurfaceInteraction;
use light::{is_delta_light, Light};
use light_arena::Allocator;
use ray::{Ray, RayDifferential};
use sampler::Sampler;
use sampling::{power_heuristic, Distribution1D};
use scene::Scene;

mod whitted;
mod directlighting;
mod path;
mod ao;
mod normal;

pub use self::whitted::Whitted;
pub use self::directlighting::{DirectLightingIntegrator, LightStrategy};
pub use self::path::PathIntegrator;
pub use self::ao::AmbientOcclusion;
pub use self::normal::Normal;

pub trait SamplerIntegrator {
    fn preprocess(&mut self, _scene: &Scene, _sampler: &mut Box<Sampler + Send + Sync>) {}

    fn li(&self,
          scene: &Scene,
          ray: &mut Ray,
          sampler: &mut Box<Sampler + Send + Sync>,
          arena: &Allocator,
          depth: u32)
          -> Spectrum;

    #[allow(non_snake_case)]
    fn specular_reflection(&self,
                           ray: &mut Ray,
                           isect: &SurfaceInteraction,
                           scene: &Scene,
                           bsdf: &bsdf::BSDF,
                           sampler: &mut Box<Sampler + Send + Sync>,
                           arena: &Allocator,
                           depth: u32)
                           -> Spectrum {
        let flags = BxDFType::BSDF_REFLECTION | BxDFType::BSDF_SPECULAR;
        let (f, wi, pdf, _bsdf_type) = bsdf.sample_f(&isect.wo, &sampler.get_2d(), flags);
        let ns = &isect.shading.n;
        if pdf > 0.0 && !f.is_black() && wi.dotn(ns).abs() != 0.0 {
            let mut r = isect.spawn_ray(&wi);
            if let Some(diff) = ray.differential {
                let mut rddiff = RayDifferential::default();
                rddiff.rx_origin = isect.p + isect.dpdx;
                rddiff.ry_origin = isect.p + isect.dpdy;
                // Compute differential reflected direction
                let dndx = isect.shading.dndu * isect.dudx + isect.dndv * isect.dvdx;
                let dndy = isect.shading.dndu * isect.dudy + isect.dndv * isect.dvdy;
                let dwodx = -diff.rx_direction - isect.wo;
                let dwody = -diff.ry_direction - isect.wo;
                let dDNdx = dwodx.dotn(ns) + isect.wo.dotn(&dndx);
                let dDNdy = dwody.dotn(ns) + isect.wo.dotn(&dndy);
                rddiff.rx_direction = wi - dwodx +
                                      2.0 * Vector3f::from(isect.wo.dotn(ns) * dndx + dDNdx * *ns);
                rddiff.ry_direction = wi - dwody +
                                      2.0 * Vector3f::from(isect.wo.dotn(ns) * dndy + dDNdy * *ns);

                r.differential = Some(rddiff);
            }
            let refl = self.li(scene, &mut r, sampler, arena, depth + 1);
            f * refl * wi.dotn(ns).abs() / pdf
        } else {
            Spectrum::black()
        }
    }

    #[allow(non_snake_case)]
    fn specular_transmission(&self,
                             ray: &mut Ray,
                             isect: &SurfaceInteraction,
                             scene: &Scene,
                             bsdf: &bsdf::BSDF,
                             sampler: &mut Box<Sampler + Send + Sync>,
                             arena: &Allocator,
                             depth: u32)
                             -> Spectrum {
        let flags = BxDFType::BSDF_TRANSMISSION | BxDFType::BSDF_SPECULAR;
        let (f, wi, pdf, _bsdf_type) = bsdf.sample_f(&isect.wo, &sampler.get_2d(), flags);
        let ns = &isect.shading.n;
        if pdf > 0.0 && !f.is_black() && wi.dotn(ns).abs() != 0.0 {
            let mut r = isect.spawn_ray(&wi);
            if let Some(diff) = ray.differential {
                let mut rddiff = RayDifferential::default();
                rddiff.rx_origin = isect.p + isect.dpdx;
                rddiff.ry_origin = isect.p + isect.dpdy;

                let mut eta = bsdf.eta;
                let w = -isect.wo;
                if isect.wo.dotn(ns) < 0.0 {
                    eta = 1.0 / eta;
                }

                // Compute differential reflected direction
                let dndx = isect.shading.dndu * isect.dudx + isect.dndv * isect.dvdx;
                let dndy = isect.shading.dndu * isect.dudy + isect.dndv * isect.dvdy;
                let dwodx = -diff.rx_direction - isect.wo;
                let dwody = -diff.ry_direction - isect.wo;
                let dDNdx = dwodx.dotn(ns) + isect.wo.dotn(&dndx);
                let dDNdy = dwody.dotn(ns) + isect.wo.dotn(&dndy);

                let mu = eta * w.dotn(ns) - wi.dotn(ns);
                let _dmudx = (eta - (eta * eta * w.dotn(ns)) / wi.dotn(ns)) * dDNdx;
                let _dmudy = (eta - (eta * eta * w.dotn(ns)) / wi.dotn(ns)) * dDNdy;

                rddiff.rx_direction = wi + eta * dwodx - Vector3f::from(mu * dndx + dDNdx * *ns);
                rddiff.ry_direction = wi + eta * dwody - Vector3f::from(mu * dndy + dDNdy * *ns);

                r.differential = Some(rddiff);
            }
            let refr = self.li(scene, &mut r, sampler, arena, depth + 1);
            f * refr * wi.dotn(ns).abs() / pdf
        } else {
            Spectrum::black()
        }
    }
}

pub fn uniform_sample_all_light(it: &SurfaceInteraction,
                                scene: &Scene,
                                sampler: &mut Box<Sampler + Send + Sync>,
                                n_light_samples: &[usize])
                                -> Spectrum {
    let mut L = Spectrum::black();
    for j in 0..scene.lights.len() {
        // Accumulate contribution of j_th light to L
        let light = &scene.lights[j];
        let n_samples = n_light_samples[j];
        // FIXME find a way to not copy the arrays into a vec...
        let u_light_array = sampler.get_2d_array(n_samples).map(|a| a.to_vec());
        let u_scattering_array = sampler.get_2d_array(n_samples).map(|a| a.to_vec());
        if u_scattering_array.is_none() || u_light_array.is_none() {
            // Use a single sample for illumination from light
            let u_light = sampler.get_2d();
            let u_scattering = sampler.get_2d();
            L += estimate_direct(it, &u_scattering, light, &u_light, scene, sampler);
        } else {
            let u_light_array = u_light_array.unwrap();
            let u_scattering_array = u_scattering_array.unwrap();
            let mut Ld = Spectrum::black();
            for k in 0..n_samples {
                Ld += estimate_direct(it,
                                      &u_scattering_array[k],
                                      light,
                                      &u_light_array[k],
                                      scene,
                                      sampler);
            }
            L += Ld / n_samples as f32;
        }
    }

    L
}

pub fn uniform_sample_one_light<'a, D: Into<Option<&'a Distribution1D>>>(
    it: &SurfaceInteraction,
    scene: &Scene,
    sampler: &mut Box<Sampler + Send + Sync>,
    distrib: D,
) -> Spectrum{
    let distrib = distrib.into();
    let n_lights = scene.lights.len();
    if n_lights == 0 {
        Spectrum::black()
    } else {
        // Randomly chose a light to sample
        let s = sampler.get_1d();
        let (light_num, light_pdf) = match distrib {
            Some(distrib) => distrib.sample_discrete(s),
            None => (cmp::min(n_lights - 1, (s * n_lights as f32) as usize), 1.0 / n_lights as f32),
        };

        debug!("sampler.get_1d()={}, n_lights={}, light_num={}, light_pdf={}",
               s,
               n_lights,
               light_num,
               light_pdf);

        if light_pdf == 0.0 {
            return Spectrum::black();
        }
        let light = &scene.lights[light_num];
        let u_light = sampler.get_2d();
        let u_scattering = sampler.get_2d();
        estimate_direct(it, &u_scattering, light, &u_light, scene, sampler) / light_pdf
    }
}

pub fn estimate_direct(it: &SurfaceInteraction,
                       u_scattering: &Point2f,
                       light: &Arc<Light + Send + Sync>,
                       u_light: &Point2f,
                       scene: &Scene,
                       _sampler: &mut Box<Sampler + Send + Sync>)
                       -> Spectrum {
    let specular = false;

    let bsdf_flags = if specular {
        BxDFType::all()
    } else {
        BxDFType::all() & !BxDFType::BSDF_SPECULAR
    };
    let mut ld = Spectrum::black();
    // Sample light with multiple importance sampling
    let bsdf = it.bsdf
        .as_ref()
        .expect("There should be a BSDF set at this point!");
    let (mut li, wi, light_pdf, vis) = light.sample_li(it, u_light);
    // info!(
    //     "EstimateDirect u_light: {} -> Li: {}, wi: {}, pdf: {}",
    //     u_light,
    //     li,
    //     wi,
    //     light_pdf
    // );
    if light_pdf > 0.0 && !li.is_black() {
        // Compute BSDF for light sample
        let f = bsdf.f(&it.wo, &wi, bsdf_flags) * wi.dotn(&it.shading.n).abs();
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
                    ld += f * li * weight / light_pdf;
                }
            }
        }
        // TODO compute phase function for medium interaction when supported
    }
    // Sample BSDF with multiple importance sampling
    if !is_delta_light(light.flags()) {
        let (mut f, wi, scattering_pdf, sampled_type) =
            bsdf.sample_f(&it.wo, u_scattering, bsdf_flags);
        f *= wi.dotn(&it.shading.n).abs();
        let sampled_specular = sampled_type.contains(BxDFType::BSDF_SPECULAR);
        // TODO compute medium interaction when supported
        if !f.is_black() && scattering_pdf > 0.0 {
            // Account for light contribution along sampled direction wi
            let weight = if !sampled_specular {
                let light_pdf = light.pdf_li(it, &wi);
                if light_pdf == 0.0 {
                    return ld;
                }
                power_heuristic(1, scattering_pdf, 1, light_pdf)
            } else {
                1.0
            };

            // Find intersection and compute transmittance
            let mut ray = it.spawn_ray(&wi);
            let li = match scene.intersect(&mut ray) {
                Some(light_isect) => {
                    // Add light contribution from material sampling
                    if let Some(area_light) = light_isect.primitive.and_then(|p| p.area_light()) {
                        // let pa = &*area_light as *const _ as *const usize;
                        // let pl = &*light as *const _ as *const usize;
                        // if pa == pl {
                        if area_light.id() == light.id() {
                            // info!("  Lights are equal");
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
