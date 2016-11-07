use super::{Shape, SurfaceInteraction, Bounds3f};
use ::{Transform, Point};
use ray::Ray;

use na::{self, Inverse};

use std::f32::consts;

pub struct Sphere {
    object_to_world: Transform,
    world_to_object: Transform,
    radius: f32,
    z_min: f32,
    z_max: f32,
    theta_min: f32,
    theta_max: f32,
    phi_max: f32,
}

impl Sphere {
    pub fn new(object_to_world: Transform,
               radius: f32,
               z_min: f32,
               z_max: f32,
               phi_max: f32)
               -> Self {
        let zmin = f32::min(z_min, z_max);
        let zmax = f32::max(z_min, z_max);
        Sphere {
            object_to_world: object_to_world,
            world_to_object: object_to_world.inverse().unwrap(),
            radius: radius,
            z_min: na::clamp(zmin, -radius, radius),
            z_max: na::clamp(zmax, -radius, radius),
            theta_min: na::clamp(z_min / radius, -1.0, 1.0).acos(),
            theta_max: na::clamp(z_max / radius, -1.0, 1.0).acos(),
            phi_max: na::clamp(phi_max, 0.0, 360.0).to_radians(),
        }


    }
}

impl Shape for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<SurfaceInteraction> {
        None
    }

    fn object_bounds(&self) -> Bounds3f {
        Bounds3f {
            p_min: Point::new(-self.radius, -self.radius, self.z_min),
            p_max: Point::new(self.radius, self.radius, self.z_max),
        }
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Sphere::new(na::one(), 1.0, -1.0, 1.0, 360.0)
    }
}
