use std::f32::consts;

use na;

use {Point3f, Vector3f, Transform, Point2f};
use bounds::Bounds3f;
use interaction::{Interaction, SurfaceInteraction};
use ray::Ray;
use sampling::concentric_sample_disk;
use shapes::Shape;

pub struct Disk {
    height: f32,
    radius: f32,
    inner_radius: f32,
    phi_max: f32,
    object_to_world: Transform,
    world_to_object: Transform,
}

impl Disk {
    pub fn new(height: f32,
               radius: f32,
               inner_radius: f32,
               phi_max: f32,
               object_to_world: Transform)
               -> Disk {
        assert!(radius > 0.0 && inner_radius >= 0.0 && phi_max > 0.0);
        Disk {
            height: height,
            radius: radius,
            inner_radius: inner_radius,
            phi_max: na::clamp(phi_max, 0.0, 360.0).to_radians(),
            world_to_object: object_to_world.inverse(),
            object_to_world: object_to_world,
        }
    }
}

impl Shape for Disk {
    fn intersect(&self, r: &Ray) -> Option<(SurfaceInteraction, f32)> {
        // Transform ray to object space
        let (ray, _o_err, _d_err) = r.transform(&self.world_to_object);
        // Compute plane intersection for disk
        if ray.d.z == 0.0 {
            // Reject disk intersection for rays parallel to the disk plane
            return None;
        }
        let t_shape_hit = (self.height - ray.o.z) / ray.d.z;
        if t_shape_hit <= 0.0 || t_shape_hit > ray.t_max {
            return None;
        }
        // See if hit point is inside radii and phi_max
        let mut p_hit = ray.at(t_shape_hit);
        let dist2 = p_hit.x * p_hit.x + p_hit.y * p_hit.y;
        if dist2 > self.radius * self.radius || dist2 < self.inner_radius * self.inner_radius {
            return None;
        }
        let mut phi = p_hit.y.atan2(p_hit.x);
        if phi < 0.0 {
            phi += 2.0 * consts::PI;
        }
        if phi > self.phi_max {
            return None;
        }
        // Find parametric representation of disk hit
        let u = phi / self.phi_max;
        let r_hit = dist2.sqrt();
        let one_minus_v = (r_hit - self.inner_radius) / (self.radius - self.inner_radius);
        let v = 1.0 - one_minus_v;
        let dpdu = Vector3f::new(-self.phi_max * p_hit.y, self.phi_max * p_hit.x, 0.0);
        let dpdv = Vector3f::new(p_hit.x, p_hit.y, 0.0) * (self.inner_radius - self.radius) / r_hit;
        // Refine disk intersection point
        p_hit.z = self.height;
        // Compute error bounds for intersection point
        let p_err = Vector3f::new(0.0, 0.0, 0.0);
        // Initialize SurfaceInteraction from parametric information
        let isect =
            SurfaceInteraction::new(p_hit, p_err, Point2f::new(u, v), -ray.d, dpdu, dpdv, self);
        // Update t_hit for quadric intersection

        Some((isect.transform(&self.object_to_world), t_shape_hit))
    }

    fn object_bounds(&self) -> Bounds3f {
        Bounds3f::from_points(&Point3f::new(-self.radius, -self.radius, self.height),
                              &Point3f::new(self.radius, self.radius, self.height))
    }

    fn world_bounds(&self) -> Bounds3f {
        let ob = self.object_bounds();
        let p1 = &self.object_to_world * &ob.p_min;
        let p2 = &self.object_to_world * &ob.p_max;
        let p_min = Point3f::new(p1.x.min(p2.x), p1.y.min(p2.y), p1.z.min(p2.z));
        let p_max = Point3f::new(p1.x.max(p2.x), p1.y.max(p2.y), p1.z.max(p2.z));
        Bounds3f::from_points(&p_min, &p_max)
    }

    fn sample(&self, u: &Point2f) -> (Interaction, f32) {
        let pd = concentric_sample_disk(u);
        let p_obj = Point3f::new(pd.x * self.radius, pd.y * self.radius, self.height);
        let n = self.object_to_world.transform_normal(&Vector3f::z()).normalize();
        let (p, p_err) =
            self.object_to_world.transform_point_with_error(&p_obj, &Vector3f::new(0.0, 0.0, 0.0));
        let pdf = 1.0 / self.area();

        (Interaction::new(p, p_err, Vector3f::new(0.0, 0.0, 0.0), n), pdf)
    }

    fn area(&self) -> f32 {
        self.phi_max * 0.5 * (self.radius * self.radius - self.inner_radius * self.inner_radius)
    }
}
