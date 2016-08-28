use std::f32::consts::*;

use scene::Scene;
use ray::Ray;
use colour::Colourf;
use integrator::Integrator;
use na::Dot;

pub struct Whitted {
    pub max_ray_depth: u8,
}

impl Whitted {
    pub fn new(n: u8) -> Whitted {
        Whitted { max_ray_depth: n }
    }
}

impl Integrator for Whitted {
    fn illumination(&self, scene: &Scene, ray: &mut Ray) -> Colourf {
        let bias = 1e-4;
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
            for light in &scene.lights {
                let shading_info = light.shading_info(&p);
//                println!("{:?}, n={:?}", shading_info, n);
                let mut shadow_ray = ray.spawn(p + bias * n, shading_info.w_i);
                shadow_ray.t_max = shading_info.light_distance;
                if let None = scene.intersect(&mut shadow_ray) {
                    let diffuse = mat.surface_colour / PI * shading_info.l_i * shading_info.w_i.dot(&n).max(0.0);
               // println!("diffuse = {:?}", diffuse);
                    colour += diffuse;
                }
            }
        } else {
            return Colourf::rgb(0.0, 0.0, 0.5);
        }

        return colour;
    }
}
