use {Point, Vector};
use ray::Ray;
use geometry::*;
use na::{Norm, Dot};
use na::clamp;
use std::f32::consts::{PI, FRAC_1_PI};

#[derive(Debug)]
pub struct Sphere {
    radius: f32,
    radius_2: f32,
    bounds: BBox,
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
            bounds: BBox::from_points(&Point::new(-r, -r, -r), &Point::new(r, r, r)),
        }
    }

    pub fn intersect_sphere(&self, ray: &Ray) -> Option<(f32, f32)> {
        let l = ray.o.to_vector();
        let a = ray.d.dot(&ray.d);
        let b = 2.0 * ray.d.dot(&l);
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

            let phi_max = 2.0 * PI;
            let phit = ray.at(ray.t_max);
            let nhit = phit.to_vector().normalize();
            let mut phi = f32::atan2(phit.y, phit.x);
            if phi < 0.0 {
                phi += 2.0 * PI;
            }
            let theta = f32::acos(clamp(phit.z / self.radius, -1.0, 1.0));
            // Compute parameterization coordinates
            let u = phi / phi_max;
            let v = theta * FRAC_1_PI;
            // Compute dpdu and dpdv
            let z_radius = (phit.x * phit.x + phit.y * phit.y).sqrt();
            let inv_z_radius = 1.0 / z_radius;
            let cos_phi = phit.x * inv_z_radius;
            let sin_phi = phit.y * inv_z_radius;
            let dpdu = Vector::new(-phi_max * phit.y, phi_max * phit.x, 0.0);
            let dpdv = PI *
                       Vector::new(phit.z * cos_phi,
                                   phit.z * sin_phi,
                                   -self.radius * theta.sin());
            Some(DifferentialGeometry::new(phit,
                                           nhit,
                                           dpdu,
                                           dpdv,
                                           TextureCoordinate { u: u, v: v },
                                           self))
        })

    }
}

impl Bounded for Sphere {
    fn get_world_bounds(&self) -> BBox {
        self.bounds
    }
}
