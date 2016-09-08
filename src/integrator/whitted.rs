use std::f32::consts::*;

use Vector;
use scene::Scene;
use ray::Ray;
use colour::Colourf;
use integrator::Integrator;
use geometry::TextureCoordinate;
use na::{Norm, Dot};
use na;

pub struct Whitted {
    pub max_ray_depth: u8,
}

impl Whitted {
    pub fn new(n: u8) -> Whitted {
        Whitted { max_ray_depth: n }
    }

    fn reflect(&self, i: &Vector, n: &Vector) -> Vector {
        (*i - *n * 2.0 * n.dot(i)).normalize()
    }
}

fn fmod(x: f32) -> f32 {
    x - x.floor()
}

fn pattern(tex_coord: &TextureCoordinate, scale_u: f32, scale_v: f32) -> f32 {
    let p = (fmod(tex_coord.u * scale_u) < 0.5) ^ (fmod(tex_coord.v * scale_v) < 0.5);
    if p { 1.0 } else {0.5}
}

impl Integrator for Whitted {
    fn illumination(&self, scene: &Scene, ray: &mut Ray) -> Colourf {
        let mut colour = Colourf::black();

        if ray.depth > self.max_ray_depth {
            return Colourf::rgb(0.0, 0.0, 0.5);
        }

        if let Some(intersection) = scene.intersect(ray) {
            let hit = intersection.hit;
            let ref mat = hit.material;
            let w_o = -ray.dir;
            let n = intersection.dg.nhit;
            let p = intersection.dg.phit;

            // colour += Colourf::grey(w_o.dot(&n));
            if mat.reflection == 0.0 && mat.transparency == 0.0 {
                // Diffuse material
                for light in &scene.lights {
                    let shading_info = light.shading_info(&p);
                    let mut shadow_ray = ray.spawn(p, shading_info.w_i);
                    shadow_ray.t_max = shading_info.light_distance;
                    if let None = scene.intersect(&mut shadow_ray) {
                        let diffuse = mat.surface_colour * FRAC_1_PI * shading_info.l_i * shading_info.w_i.dot(&n).max(0.0) * pattern(&intersection.dg.tex_coord, 10.0, 10.0);
                        colour += diffuse;
                    }
                }
            }

            if mat.reflection > 0.0 {
                let mut refl_ray = ray.spawn(p, self.reflect(&ray.dir, &n));
                let refl = self.illumination(scene, &mut refl_ray);
                colour += refl * mat.reflection;
                if mat.transparency > 0.0 {
                    let mut cos_i = na::clamp(ray.dir.dot(&n), -1.0, 1.0);
                    let mut eta = 1.5;
                    let mut n_refl = n;
                    if cos_i < 0.0 {
                        cos_i = -cos_i;
                    } else {
                        eta = 1.0 / eta;
                        n_refl = -n_refl;
                    }
                    let k = 1.0 - eta * eta * (1.0 - cos_i * cos_i);
                    if k > 0.0 {
                        let refl_dir = eta * ray.dir + (eta * cos_i - k.sqrt()) * n_refl;
                        let mut refl_ray = ray.spawn(p, refl_dir.normalize());
                        colour += self.illumination(scene, &mut refl_ray) * mat.transparency;
                    }
                }
            }
        } else {
            return Colourf::rgb(0.0, 0.0, 0.5);
        }

        return colour;
    }
}
