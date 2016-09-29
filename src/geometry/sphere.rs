use ray::Ray;
use geometry::*;
use na::{Norm, Dot};
use na::clamp;
use std::f32::consts::FRAC_1_PI;

#[derive(Debug, PartialEq)]
pub struct Sphere {
    radius: f32,
    radius_2: f32,
}

fn solve_quadratic(a: f32, b: f32, c: f32) -> Option<(f32, f32)> {
    let discr = b * b - 4.0 * a * c;

    if discr < 0.0 {
        None
    } else if discr == 0.0 {
        let x = -0.5 * b / a;
        Some((x, x))
    } else {
        let q = if b > 0.0 {
            -0.5 * (b + discr.sqrt())
        } else {
            -0.5 * (b - discr.sqrt())
        };
        let x0 = q / a;
        let x1 = c / q;

        if x0 > x1 {
            Some((x1, x0))
        } else {
            Some((x0, x1))
        }
    }
}

impl Sphere {
    pub fn new(r: f32) -> Sphere {
        Sphere {
            radius: r,
            radius_2: r * r,
        }
    }

    pub fn intersect_sphere(&self, ray: &Ray) -> Option<(f32, f32)> {
        let l = ray.origin.to_vector();
        let a = ray.dir.dot(&ray.dir); // 1.0;
        let b = 2.0 * ray.dir.dot(&l);
        let c = l.dot(&l) - self.radius_2;

        solve_quadratic(a, b, c)
    }
}

impl Geometry for Sphere {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        self.intersect_sphere(ray).and_then(|(t0, t1)| {
            if t1 < ray.t_min || t0 > ray.t_max {
                return None;
            }

            let t_hit;
            if t0 >= ray.t_min {
                t_hit = t0;
            } else if t1 <= ray.t_max {
                t_hit = t1;
            } else {
                return None;
            }

            ray.t_max = t_hit;

            let phit = ray.at(ray.t_max);
            let nhit = phit.to_vector().normalize();
            let phi = f32::atan2(phit.z, phit.x);
            let theta = f32::acos(clamp(phit.y / self.radius, -1.0, 1.0));
            let u = if phi < 0.0 {
                phi * FRAC_1_PI + 1.0
            } else {
                phi * FRAC_1_PI
            };
            let v = theta * FRAC_1_PI;
            Some(DifferentialGeometry::new(phit, nhit, TextureCoordinate { u: u, v: v }, self))
        })

    }
}
