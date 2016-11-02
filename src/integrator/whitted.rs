use std::f32::consts::*;

use Vector;
use scene::Scene;
use ray::Ray;
use colour::Colourf;
use integrator::Integrator;
use geometry::TextureCoordinate;
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

impl Integrator for Whitted {
    fn illumination(&self, scene: &Scene, ray: &mut Ray) -> Colourf {
        let mut colour = Colourf::black();

        if ray.depth > self.max_ray_depth {
            return Colourf::rgb(0.0, 0.0, 0.5);
        }

        if let Some(intersection) = scene.intersect(ray) {
            let hit = intersection.hit;
            let mat = &hit.material;
            let n = intersection.dg.nhit;
            let p = intersection.dg.phit;

            if mat.reflection == 0.0 && mat.transparency == 0.0 {
                // Diffuse material
                for light in &scene.lights {
                    let shading_info = light.shading_info(&p);
                    let mut shadow_ray = ray.spawn(p, shading_info.w_i);
                    shadow_ray.t_max = shading_info.light_distance;
                    if !scene.intersect_p(&mut shadow_ray) {
                        let diffuse = mat.surface_colour * FRAC_1_PI * shading_info.l_i *
                                      shading_info.w_i.dot(&n).max(0.0) *
                                      pattern(&intersection.dg.tex_coord, 10.0, 10.0);
                        colour += diffuse;
                    }
                }
            } else {
                // Fresnel reflection / refraction
                let kr = fresnel(&ray.dir, &n, 1.5);
                let bias = if ray.dir.dot(&n) < 0.0 {
                    // outside
                    1e-4 * n
                } else {
                    // inside
                    -1e-4 * n
                };

                if kr < 1.0 {
                    // refraction
                    let refr_dir = refract(&ray.dir, &n, 1.5);
                    let mut refr_ray = ray.spawn(p - bias, refr_dir);
                    let refr = self.illumination(scene, &mut refr_ray) * (1.0 - kr);
                    colour += refr;
                }
                // Reflection
                let mut refl_ray = ray.spawn(p + bias, reflect(&ray.dir, &n));
                let refl = self.illumination(scene, &mut refl_ray);
                colour += refl * kr;
            }

        } else {
            return scene.atmosphere.compute_incident_light(ray);
        }

        colour
    }
}
