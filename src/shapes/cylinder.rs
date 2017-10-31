use std::sync::Arc;

use num::zero;

use {clamp, gamma, lerp, Normal3f, Point2f, Point3f, Transform, Vector3f};
use bounds::Bounds3f;
use efloat::{solve_quadratic, EFloat};
use interaction::{Interaction, SurfaceInteraction};
use paramset::ParamSet;
use ray::Ray;
use shapes::Shape;

#[derive(Debug)]
pub struct Cylinder {
    object_to_world: Transform,
    world_to_object: Transform,
    radius: f32,
    z_min: f32,
    z_max: f32,
    phi_max: f32,
    reverse_orientation: bool,
    transform_swaps_handedness: bool,
}

impl Cylinder {
    pub fn create(object_to_world: &Transform,
                  reverse_orientation: bool,
                  params: &mut ParamSet)
                  -> Arc<Shape + Send + Sync> {
        let radius = params.find_one_float("radius", 1.0);
        let z_min = params.find_one_float("z_min", -1.0);
        let z_max = params.find_one_float("z_max", 1.0);
        let phi_max = params.find_one_float("phi_max", 360.0);

        Arc::new(Cylinder {
                     object_to_world: object_to_world.clone(),
                     world_to_object: object_to_world.inverse(),
                     radius,
                     z_min,
                     z_max,
                     phi_max: clamp(phi_max, 0.0, 360.0).to_radians(),
                     reverse_orientation,
                     transform_swaps_handedness: object_to_world.swaps_handedness(),
                 })
    }
}

impl Shape for Cylinder {
    fn object_bounds(&self) -> Bounds3f {
        Bounds3f::from_points(&Point3f::new(-self.radius, -self.radius, self.z_min),
                              &Point3f::new(self.radius, self.radius, self.z_max))
    }

    fn world_bounds(&self) -> Bounds3f {
        &self.object_to_world * &self.object_bounds()
    }

    #[allow(non_snake_case)]
    fn intersect(&self, r: &Ray) -> Option<(SurfaceInteraction, f32)> {
        // Transform ray to object space
        let (ray, o_err, d_err) = r.transform(&self.world_to_object);

        // Compute quadratic cylinder coefficients

        // Initialize EFloat ray coordinate values
        let ox = EFloat::new(ray.o.x, o_err.x);
        let oy = EFloat::new(ray.o.y, o_err.y);
        let _oz = EFloat::new(ray.o.z, o_err.z);
        let dx = EFloat::new(ray.d.x, d_err.x);
        let dy = EFloat::new(ray.d.y, d_err.y);
        let _dz = EFloat::new(ray.d.z, d_err.z);
        let a = dx * dx + dy * dy;
        let b = 2.0 * (dx * ox + dy * oy);
        let c = ox * ox + oy * oy - EFloat::from(self.radius) * EFloat::from(self.radius);

        // Solve quadratic equation for t values
        if let Some((t0, t1)) = solve_quadratic(&a, &b, &c) {
            if t0.upper_bound() > ray.t_max || t1.lower_bound() <= 0.0 {
                return None;
            }
            let mut t_shape_hit = t0;
            if t_shape_hit.lower_bound() <= 0.0 {
                t_shape_hit = t1;
                if t_shape_hit.upper_bound() > ray.t_max {
                    return None;
                }
            }

            // Compute cylinder hit point and phi
            let mut p_hit = ray.at(t_shape_hit.into());

            // Refine cylinder intersection point
            let hit_rad = (p_hit.x * p_hit.x + p_hit.y * p_hit.y).sqrt();
            p_hit.x *= self.radius / hit_rad;
            p_hit.y *= self.radius / hit_rad;
            let mut phi = p_hit.y.atan2(p_hit.x);
            if phi < 0.0 {
                phi += 2.0 * ::std::f32::consts::PI;
            }

            // Test cylinder intersection against clipping parameters
            if p_hit.z < self.z_min || p_hit.z > self.z_max || phi > self.phi_max {
                if t_shape_hit == t1 {
                    return None;
                }
                t_shape_hit = t1;
                if t1.upper_bound() > ray.t_max {
                    return None;
                }
                // Compute cylinder hit point and phi
                p_hit = ray.at(t_shape_hit.into());

                // Refine cylinder intersection point
                let hit_rad = (p_hit.x * p_hit.x + p_hit.y * p_hit.y).sqrt();
                p_hit.x *= self.radius / hit_rad;
                p_hit.y *= self.radius / hit_rad;
                phi = p_hit.y.atan2(p_hit.x);
                if phi < 0.0 {
                    phi += 2.0 * ::std::f32::consts::PI;
                }
                if p_hit.z < self.z_min || p_hit.z > self.z_max || phi > self.phi_max {
                    return None;
                }
            }

            // Find parametric representation of cylinder hit
            let u = phi / self.phi_max;
            let v = (p_hit.z - self.z_min) / (self.z_max / self.z_min);

            // Compute cylinder dpdu and dpdv
            let dpdu = Vector3f::new(-self.phi_max * p_hit.y, self.phi_max * p_hit.x, 0.0);
            let dpdv = Vector3f::new(0.0, 0.0, self.z_max - self.z_min);

            // Compute cylinder dndu and dndv
            let d2Pduu = -self.phi_max * self.phi_max * Vector3f::new(p_hit.x, p_hit.y, 0.0);
            let d2Pduv = Vector3f::new(0.0, 0.0, 0.0);
            let d2Pdvv = Vector3f::new(0.0, 0.0, 0.0);

            // Compute coefficients for fundamental forms
            let E = dpdu.dot(&dpdu);
            let F = dpdu.dot(&dpdv);
            let G = dpdv.dot(&dpdv);
            let N = dpdu.cross(&dpdv).normalize();
            let e = N.dot(&d2Pduu);
            let f = N.dot(&d2Pduv);
            let g = N.dot(&d2Pdvv);

            // Compute dndu and dndv from fundamental form coefficients
            let inv_EGF2 = 1.0 / (E * G - F * F);
            let _dndu = Normal3f::from((f * F - e * G) * inv_EGF2 * dpdu +
                                       (e * F - f * E) * inv_EGF2 * dpdv);
            let _dndv = Normal3f::from((g * F - f * G) * inv_EGF2 * dpdu +
                                       (f * F - g * E) * inv_EGF2 * dpdv);

            let p_error = gamma(3) * Vector3f::new(p_hit.x.abs(), p_hit.y.abs(), 0.0);

            let isect = SurfaceInteraction::new(p_hit,
                                                p_error,
                                                Point2f::new(u, v),
                                                -ray.d,
                                                dpdu,
                                                dpdv,
                                                self);

            Some((isect, t_shape_hit.into()))
        } else {
            None
        }
    }

    // TODO specialize intersect_p()

    fn area(&self) -> f32 {
        (self.z_max - self.z_min) * self.radius * self.phi_max
    }


    fn sample(&self, u: &Point2f) -> (Interaction, f32) {
        let z = lerp(u[0], self.z_min, self.z_max);
        let phi = u[1] * self.phi_max;
        let mut p_obj = Point3f::new(self.radius * phi.cos(), self.radius * phi.sin(), z);
        let mut n = (&self.object_to_world * &Normal3f::new(p_obj.x, p_obj.y, 0.0)).normalize();
        if self.reverse_orientation {
            n *= -1.0;
        }
        // Reproject p_obj to cylinder surface and compute p_obj_error
        let hit_rad = (p_obj.x * p_obj.x + p_obj.y * p_obj.y).sqrt();
        p_obj.x *= self.radius / hit_rad;
        p_obj.y *= self.radius / hit_rad;
        let p_obj_error = gamma(3) * Vector3f::new(p_obj.x.abs(), p_obj.y.abs(), 0.0);
        let (p, p_error) = self.object_to_world
            .transform_point_with_error(&p_obj, &p_obj_error);

        let it = Interaction::new(p, p_error, zero(), n);
        (it, 1.0 / self.area())
    }

    fn reverse_orientation(&self) -> bool {
        self.reverse_orientation
    }

    fn transform_swaps_handedness(&self) -> bool {
        self.transform_swaps_handedness
    }
}
