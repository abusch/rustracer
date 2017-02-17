use std::f32::consts;

use na::{self, one};

use {gamma, Transform, Point2f, Point3f, Vector3f};
use bounds::Bounds3f;
use efloat::{self, EFloat};
use interaction::{Interaction, SurfaceInteraction};
use ray::Ray;
use sampling::uniform_sample_sphere;
use shapes::Shape;
use transform::{transform_point_with_error, transform_normal};

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
    pub fn new() -> Self {
        Sphere {
            object_to_world: one(),
            world_to_object: one(),
            radius: 1.0,
            z_min: -1.0,
            z_max: 1.0,
            theta_min: consts::PI,
            theta_max: 0.0,
            phi_max: 2.0 * consts::PI,
        }
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self.z_min = -radius;
        self.z_max = radius;
        self.theta_min = consts::PI;
        self.theta_max = 0.0;

        self
    }

    pub fn z_min(mut self, z_min: f32) -> Self {
        assert!(z_min <= self.z_max);
        self.z_min = na::clamp(z_min, -self.radius, self.radius);
        self.theta_min = na::clamp(z_min / self.radius, -1.0, 1.0).acos();

        self
    }

    pub fn z_max(mut self, z_max: f32) -> Self {
        assert!(self.z_min <= z_max);
        self.z_max = na::clamp(z_max, -self.radius, self.radius);
        self.theta_max = na::clamp(z_max / self.radius, -1.0, 1.0).acos();

        self
    }

    pub fn phi_max(mut self, phi_max: f32) -> Self {
        self.phi_max = na::clamp(phi_max, 0.0, 360.0).to_radians();

        self
    }

    pub fn transform(mut self, object_to_world: Transform) -> Self {
        self.object_to_world = object_to_world;
        self.world_to_object = object_to_world.inverse();

        self
    }
}

impl Shape for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<(SurfaceInteraction, f32)> {
        // Transform ray into object space
        let (r, o_err, d_err) = ray.transform(&self.world_to_object);

        // Compute quadratic coefficients
        let ox = EFloat::new(r.o.x, o_err.x);
        let oy = EFloat::new(r.o.y, o_err.y);
        let oz = EFloat::new(r.o.z, o_err.z);
        let dx = EFloat::new(r.d.x, d_err.x);
        let dy = EFloat::new(r.d.y, d_err.y);
        let dz = EFloat::new(r.d.z, d_err.z);
        let a = dx * dx + dy * dy + dz * dz;
        let b = 2.0 * (dx * ox + dy * oy + dz * oz);
        let c = (ox * ox + oy * oy + oz * oz) -
                EFloat::from(self.radius) * EFloat::from(self.radius);

        // Solve quadratic equation for t values
        efloat::solve_quadratic(&a, &b, &c).and_then(|(t0, t1)| {
            if t0.upper_bound() > r.t_max || t1.lower_bound() <= 0.0 {
                return None;
            }
            // Check quadric shape t0 and t1 for nearest intersection
            let mut t_shape_hit = t0;
            if t_shape_hit.lower_bound() <= 0.0 {
                t_shape_hit = t1;
                if t_shape_hit.upper_bound() > r.t_max {
                    return None;
                }
            }

            // Compute sphere hit position and phi
            let mut p_hit = r.at(t_shape_hit.into());
            // Refine sphere intersection point
            if p_hit.x == 0.0 && p_hit.y == 0.0 {
                p_hit.x = 1e-5 * self.radius;
            }
            p_hit *= self.radius / p_hit.coords.norm();
            let mut phi = f32::atan2(p_hit.x, p_hit.y);
            if phi < 0.0 {
                phi += 2.0 * consts::PI;
            }
            // Test intersection against clipping parameters
            if (self.z_min > -self.radius && p_hit.z < self.z_min) ||
               (self.z_max < self.radius && p_hit.z > self.z_max) ||
               phi > self.phi_max {
                if t_shape_hit == t1 {
                    return None;
                }

                // Try again with t1
                t_shape_hit = t1;
                // Compute sphere hit position and phi
                p_hit = r.at(t_shape_hit.into());
                // Refine sphere intersection point
                if p_hit.x == 0.0 && p_hit.y == 0.0 {
                    p_hit.x = 1e-5 * self.radius;
                }
                p_hit *= self.radius / p_hit.coords.norm();
                phi = f32::atan2(p_hit.x, p_hit.y);
                if phi < 0.0 {
                    phi += 2.0 * consts::PI;
                }
                if (self.z_min > -self.radius && p_hit.z < self.z_min) ||
                   (self.z_max < self.radius && p_hit.z > self.z_max) ||
                   phi > self.phi_max {
                    return None;
                }
            }
            // Find parametric representation of sphere hit
            let u = phi / self.phi_max;
            let theta = na::clamp(p_hit.z / self.radius, -1.0, 1.0).acos();
            let v = (theta - self.theta_min) / (self.theta_max - self.theta_min);
            // Compute error bound for sphere intersection
            let p_error = gamma(5) * p_hit.coords.abs();
            // Compute dp/du and dp/dv
            let z_radius = (p_hit.x * p_hit.x + p_hit.y * p_hit.y).sqrt();
            let inv_z_radius = 1.0 / z_radius;
            let cos_phi = p_hit.x * inv_z_radius;
            let sin_phi = p_hit.y * inv_z_radius;
            let dpdu = Vector3f::new(-self.phi_max * p_hit.y, self.phi_max * p_hit.x, 0.0);
            let dpdv = (self.theta_max - self.theta_min) *
                       Vector3f::new(p_hit.z * cos_phi,
                                     p_hit.z * sin_phi,
                                     -self.radius * theta.sin());
            // TODO Compute dn/du and dn/dv
            let isect =
                SurfaceInteraction::new(p_hit, p_error, Point2f::new(u, v), -r.d, dpdu, dpdv, self);
            Some((isect.transform(&self.object_to_world), t_shape_hit.into()))
        })
    }

    fn object_bounds(&self) -> Bounds3f {
        Bounds3f {
            p_min: Point3f::new(-self.radius, -self.radius, self.z_min),
            p_max: Point3f::new(self.radius, self.radius, self.z_max),
        }
    }

    fn world_bounds(&self) -> Bounds3f {
        let mut bounds = Bounds3f::new();
        let b = self.object_bounds();
        bounds.extend(self.object_to_world * Point3f::new(b[0].x, b[0].y, b[0].z));
        bounds.extend(self.object_to_world * Point3f::new(b[1].x, b[0].y, b[0].z));
        bounds.extend(self.object_to_world * Point3f::new(b[0].x, b[1].y, b[0].z));
        bounds.extend(self.object_to_world * Point3f::new(b[0].x, b[0].y, b[1].z));
        bounds.extend(self.object_to_world * Point3f::new(b[1].x, b[1].y, b[0].z));
        bounds.extend(self.object_to_world * Point3f::new(b[1].x, b[0].y, b[1].z));
        bounds.extend(self.object_to_world * Point3f::new(b[0].x, b[1].y, b[1].z));
        bounds.extend(self.object_to_world * Point3f::new(b[1].x, b[1].y, b[1].z));

        bounds
    }

    fn sample(&self, u: &Point2f) -> (Interaction, f32) {
        let mut p_obj = Point3f::new(0.0, 0.0, 0.0) + self.radius * uniform_sample_sphere(u);
        let n = transform_normal(&p_obj.coords, &self.object_to_world).normalize();
        p_obj = p_obj * self.radius / p_obj.coords.norm();
        let p_obj_error = gamma(5) * p_obj.coords.abs();
        let (p, p_err) = transform_point_with_error(&self.object_to_world, &p_obj, &p_obj_error);
        let pdf = 1.0 / self.area();
        (Interaction::new(p, p_err, Vector3f::new(0.0, 0.0, 0.0), n), pdf)
    }

    fn area(&self) -> f32 {
        self.phi_max * self.radius * (self.z_max - self.z_min)
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Sphere::new()
    }
}
