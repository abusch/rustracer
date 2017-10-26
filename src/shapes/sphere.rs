use std::sync::Arc;
use std::f32;
use std::f32::consts;

use {clamp, coordinate_system, gamma, Normal3f, Point2f, Point3f, Transform, Vector3f};
use bounds::Bounds3f;
use efloat::{self, EFloat};
use geometry::{distance, distance_squared, offset_ray_origin, spherical_direction_vec};
use interaction::{Interaction, SurfaceInteraction};
use paramset::ParamSet;
use ray::Ray;
use sampling::uniform_sample_sphere;
use shapes::Shape;

#[derive(Debug)]
pub struct Sphere {
    object_to_world: Transform,
    world_to_object: Transform,
    radius: f32,
    z_min: f32,
    z_max: f32,
    theta_min: f32,
    theta_max: f32,
    phi_max: f32,
    reverse_orientation: bool,
    transform_swaps_handedness: bool,
}

impl Sphere {
    pub fn new(
        o2w: Transform,
        radius: f32,
        z_min: f32,
        z_max: f32,
        phi_max: f32,
        reverse_orientation: bool,
    ) -> Self {
        let transform_swaps_handedness = o2w.swaps_handedness();
        Sphere {
            world_to_object: o2w.inverse(),
            object_to_world: o2w,
            radius,
            z_min: clamp(f32::min(z_min, z_max), -radius, radius),
            z_max: clamp(f32::max(z_min, z_max), -radius, radius),
            theta_min: f32::acos(clamp(f32::min(z_min, z_max) / radius, -1.0, 1.0)),
            theta_max: f32::acos(clamp(f32::max(z_min, z_max) / radius, -1.0, 1.0)),
            phi_max: f32::to_radians(clamp(phi_max, 0.0, 360.0)),
            reverse_orientation,
            transform_swaps_handedness,
        }
    }


    pub fn create(
        o2w: &Transform,
        reverse_orientation: bool,
        params: &mut ParamSet,
    ) -> Arc<Shape + Send + Sync> {
        let radius = params.find_one_float("radius", 1.0);
        let zmin = params.find_one_float("zmin", -radius);
        let zmax = params.find_one_float("zmax", radius);
        let phimax = params.find_one_float("phimax", 360.0);


        Arc::new(Sphere::new(
            o2w.clone(),
            radius,
            zmin,
            zmax,
            phimax,
            reverse_orientation,
        ))
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
        let c =
            (ox * ox + oy * oy + oz * oz) - EFloat::from(self.radius) * EFloat::from(self.radius);

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
            p_hit *= self.radius / Vector3f::from(p_hit).length();
            let mut phi = f32::atan2(p_hit.x, p_hit.y);
            if phi < 0.0 {
                phi += 2.0 * consts::PI;
            }
            // Test intersection against clipping parameters
            if (self.z_min > -self.radius && p_hit.z < self.z_min)
                || (self.z_max < self.radius && p_hit.z > self.z_max)
                || phi > self.phi_max
            {
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
                p_hit *= self.radius / Vector3f::from(p_hit).length();
                phi = f32::atan2(p_hit.x, p_hit.y);
                if phi < 0.0 {
                    phi += 2.0 * consts::PI;
                }
                if (self.z_min > -self.radius && p_hit.z < self.z_min)
                    || (self.z_max < self.radius && p_hit.z > self.z_max)
                    || phi > self.phi_max
                {
                    return None;
                }
            }
            // Find parametric representation of sphere hit
            let u = phi / self.phi_max;
            let theta = clamp(p_hit.z / self.radius, -1.0, 1.0).acos();
            let v = (theta - self.theta_min) / (self.theta_max - self.theta_min);
            // Compute error bound for sphere intersection
            let p_error = gamma(5) * Vector3f::from(p_hit).abs();
            // Compute dp/du and dp/dv
            let z_radius = (p_hit.x * p_hit.x + p_hit.y * p_hit.y).sqrt();
            let inv_z_radius = 1.0 / z_radius;
            let cos_phi = p_hit.x * inv_z_radius;
            let sin_phi = p_hit.y * inv_z_radius;
            let dpdu = Vector3f::new(-self.phi_max * p_hit.y, self.phi_max * p_hit.x, 0.0);
            let dpdv = (self.theta_max - self.theta_min)
                * Vector3f::new(
                    p_hit.z * cos_phi,
                    p_hit.z * sin_phi,
                    -self.radius * theta.sin(),
                );
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
        bounds.extend(&self.object_to_world * &Point3f::new(b[0].x, b[0].y, b[0].z));
        bounds.extend(&self.object_to_world * &Point3f::new(b[1].x, b[0].y, b[0].z));
        bounds.extend(&self.object_to_world * &Point3f::new(b[0].x, b[1].y, b[0].z));
        bounds.extend(&self.object_to_world * &Point3f::new(b[0].x, b[0].y, b[1].z));
        bounds.extend(&self.object_to_world * &Point3f::new(b[1].x, b[1].y, b[0].z));
        bounds.extend(&self.object_to_world * &Point3f::new(b[1].x, b[0].y, b[1].z));
        bounds.extend(&self.object_to_world * &Point3f::new(b[0].x, b[1].y, b[1].z));
        bounds.extend(&self.object_to_world * &Point3f::new(b[1].x, b[1].y, b[1].z));

        bounds
    }

    fn sample(&self, u: &Point2f) -> (Interaction, f32) {
        let mut p_obj = Point3f::new(0.0, 0.0, 0.0) + self.radius * uniform_sample_sphere(u);
        let mut it = Interaction::empty();
        it.n = self.object_to_world
            .transform_normal(&Normal3f::new(p_obj.x, p_obj.y, p_obj.z))
            .normalize();
        p_obj = p_obj * self.radius / Vector3f::from(p_obj).length();
        let p_obj_error = gamma(5) * Vector3f::from(p_obj).abs();
        let (p, p_err) = self.object_to_world
            .transform_point_with_error(&p_obj, &p_obj_error);
        it.p = p;
        it.p_error = p_err;
        let pdf = 1.0 / self.area();
        (it, pdf)
    }

    fn sample_si(&self, si: &Interaction, u: &Point2f) -> (Interaction, f32) {
        let p_center = &self.object_to_world * &Point3f::new(0.0, 0.0, 0.0);

        // Sample uniformly on sphere if `pt` is inside it
        let p_origin = offset_ray_origin(&si.p, &si.p_error, &si.n, &(p_center - si.p));
        if distance_squared(&p_origin, &p_center) <= self.radius * self.radius {
            let (intr, mut pdf) = self.sample(u);
            let mut wi = intr.p - si.p;
            if wi.length_squared() == 0.0 {
                pdf = 0.0;
            } else {
                // Convert from area measure returned by sample() call above to solid angle measure.
                wi = wi.normalize();
                pdf *= distance_squared(&si.p, &intr.p) / intr.n.dot(&(-wi)).abs();
            }

            return (intr, pdf);
        }

        // Compute coordinate system for sphere sampling
        let wc = (p_center - si.p).normalize();
        let (wc_x, wc_y) = coordinate_system(&wc);

        // Sample sphere uniformly inside subtended cone

        // Compute `theta` and `phi` values for sample in cone
        let sin_theta_max_2 = self.radius * self.radius / distance_squared(&si.p, &p_center);
        let cos_theta_max = f32::sqrt(f32::max(0.0, 1.0 - sin_theta_max_2));
        let cos_theta = (1.0 - u[0]) + u[0] * cos_theta_max;
        let sin_theta = f32::sqrt(f32::max(0.0, 1.0 - cos_theta * cos_theta));
        let phi = u[1] * 2.0 * consts::PI;

        // Compute angle `alpha` from center of sphere to sampled point on surface
        let dc = distance(&si.p, &p_center);
        let ds = dc * cos_theta
            - f32::sqrt(f32::max(
                0.0,
                self.radius * self.radius - dc * dc * sin_theta * sin_theta,
            ));
        let cos_alpha = (dc * dc + self.radius * self.radius - ds * ds) / (2.0 * dc * self.radius);
        let sin_alpha = f32::sqrt(f32::max(0.0, 1.0 - cos_alpha * cos_alpha));

        // Compute surface normal and sampled point on sphere
        let n_world =
            spherical_direction_vec(sin_alpha, cos_alpha, phi, &(-wc_x), &(-wc_y), &(-wc));
        let p_world = p_center + self.radius * n_world;

        // Return `Interaction` for sampled point on sphere
        let mut it = Interaction::empty();
        it.p = p_world;
        it.p_error = gamma(5) * Vector3f::from(p_world).abs();
        it.n = Normal3f::from(n_world);
        if self.reverse_orientation {
            it.n *= -1.0;
        }

        // Uniform cone PDF.
        let pdf = 1.0 / (2.0 * consts::PI * (1.0 - cos_theta_max));

        (it, pdf)
    }

    fn area(&self) -> f32 {
        self.phi_max * self.radius * (self.z_max - self.z_min)
    }

    fn reverse_orientation(&self) -> bool {
        self.reverse_orientation
    }

    fn transform_swaps_handedness(&self) -> bool {
        self.transform_swaps_handedness
    }
}
