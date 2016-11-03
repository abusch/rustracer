use std::f32::consts::*;

use Vector;
use colour::Colourf;
use geometry::TextureCoordinate;
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

/// Compute the reflection direction
fn reflect(i: &Vector, n: &Vector) -> Vector {
    (*i - *n * 2.0 * n.dot(i)).normalize()
}

/// Compute the refraction direction
fn refract(i: &Vector, n: &Vector, ior: f32) -> Vector {
    let mut cos_i = na::clamp(i.dot(n), -1.0, 1.0);
    let (etai, etat, n_refr) = if cos_i < 0.0 {
        cos_i = -cos_i;
        (1.0, ior, *n)
    } else {
        (ior, 1.0, -*n)
    };

    let eta = etai / etat;
    let k = 1.0 - eta * eta * (1.0 - cos_i * cos_i);

    if k > 0.0 {
        *i * eta + n_refr * (eta * cos_i - k.sqrt())
    } else {
        zero()
    }
}

/// Compute the Fresnel coefficient
fn fresnel(i: &Vector, n: &Vector, ior: f32) -> f32 {
    let mut cosi = na::clamp(i.dot(n), -1.0, 1.0);
    let (etai, etat) = if cosi > 0.0 { (ior, 1.0) } else { (1.0, ior) };

    let sint = etai / etat * (1.0 - cosi * cosi).max(0.0).sqrt();
    if sint >= 1.0 {
        1.0
    } else {
        let cost = (1.0 - sint * sint).max(0.0).sqrt();
        cosi = cosi.abs();
        let r_s = ((etat * cosi) - (etai * cost)) / ((etat * cosi) + (etai * cost));
        let r_p = ((etai * cosi) - (etat * cost)) / ((etai * cosi) + (etat * cost));
        (r_s * r_s + r_p * r_p) / 2.0
    }
}

fn fmod(x: f32) -> f32 {
    x - x.floor()
}

fn pattern(tex_coord: &TextureCoordinate, scale_u: f32, scale_v: f32) -> f32 {
    let p = (fmod(tex_coord.u * scale_u) < 0.5) ^ (fmod(tex_coord.v * scale_v) < 0.5);
    if p { 1.0 } else { 0.5 }
}

impl SamplerIntegrator for Whitted {
    fn li(&self, scene: &Scene, ray: &mut Ray, sampler: &mut Sampler) -> Colourf {
        let mut colour = Colourf::black();

        match scene.intersect(ray) {
            Some(intersection) => {
                let n = intersection.dg.nhit;
                let p = intersection.dg.phit;
                let wo = -ray.dir;

                // Compute scattering functions for surface interaction
                // TODO

                // Compute emitted light if ray hit an area light source
                colour += intersection.le(wo);

                // Add contribution of each light source
                for light in &scene.lights {
                    let (li, wi, pdf) = light.sample_li(&intersection, wo, (0.0, 0.0));
                    if li.is_black() || pdf == 0.0 {
                        continue;
                    }

                    // TODO VisibilityTester
                    let mut shadow_ray = ray.spawn(p, -wi);
                    // shadow_ray.t_max = shading_info.light_distance;
                    let f = intersection.bsdf.f(&wi, &wo);
                    if !f.is_black() && !scene.intersect_p(&mut shadow_ray) {
                        let diffuse = f * li * wi.dot(&n).abs() * FRAC_1_PI / pdf;
                        // pattern(&intersection.dg.tex_coord, 10.0, 10.0);
                        colour += diffuse;
                    }
                }

                // TODO // Fresnel reflection / refraction
                // let kr = fresnel(&ray.dir, &n, 1.5);
                // let bias = if ray.dir.dot(&n) < 0.0 {
                //     // outside
                //     1e-4 * n
                // } else {
                //     // inside
                //     -1e-4 * n
                // };

                // if kr < 1.0 {
                //     // refraction
                //     let refr_dir = refract(&ray.dir, &n, 1.5);
                //     let mut refr_ray = ray.spawn(p - bias, refr_dir);
                //     let refr = self.li(scene, &mut refr_ray) * (1.0 - kr);
                //     colour += refr;
                // }
                // // Reflection
                // let mut refl_ray = ray.spawn(p + bias, reflect(&ray.dir, &n));
                // let refl = self.li(scene, &mut refl_ray);
                // colour += refl * kr;

            }
            None => {
                return scene.atmosphere.compute_incident_light(ray);
            }
        }

        colour
    }
}
