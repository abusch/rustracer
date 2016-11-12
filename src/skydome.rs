use std::f32::consts::PI;

use Vector;
use Transform;
use ray::Ray;
use geometry::Sphere;
use colour::Colourf;
use na::{Norm, Dot, zero, Inverse};

const BETA_R: Colourf = Colourf {
    r: 5.5e-6,
    g: 13e-6,
    b: 22.4e-6,
};
const BETA_M: Colourf = Colourf {
    r: 21e-6,
    g: 21e-6,
    b: 21e-6,
};

pub struct Atmosphere {
    hr: f32,
    hm: f32,
    radius_earth: f32,
    sun_direction: Vector,
    sun_intensity: f32,
    g: f32,
    atmosphere: Sphere,
    transform_inv: Transform,
}

impl Atmosphere {
    pub fn earth(sd: Vector) -> Atmosphere {
        Atmosphere {
            hr: 7994.0,
            hm: 1200.0,
            radius_earth: 6360e3,
            sun_direction: sd,
            sun_intensity: 20.0,
            g: 0.76,
            atmosphere: Sphere::new(6420e3),
            transform_inv: Transform::new(Vector::new(0.0, -6360e3, 0.0), zero(), 1.0)
                .inverse()
                .unwrap(),
        }
    }

    pub fn compute_incident_light(&self, ray: &mut Ray) -> Colourf {
        let mut r = self.transform_inv * *ray;
        match self.atmosphere.intersect_sphere(&r) {
            None => Colourf::black(),
            Some((t0, t1)) => {
                if t1 < 0.0 {
                    return Colourf::black();
                }
                if t0 > r.t_min && t0 > 0.0 {
                    r.t_min = t0;
                }
                if t1 < r.t_max {
                    r.t_max = t1;
                }

                let num_samples = 16u8;
                let num_samples_light = 8u8;

                let segment_length = (r.t_max - r.t_min) / num_samples as f32;
                let mut t_current = r.t_min;
                let mut sum_r = Colourf::black();
                let mut sum_m = Colourf::black();
                let mut optical_depth_r = 0.0;
                let mut optical_depth_m = 0.0;
                let mu = r.d.normalize().dot(&self.sun_direction);
                let phase_r = 3.0 / (16.0 * PI) * (1.0 + mu * mu);
                let phase_m = 3.0 / (8.0 * PI) * ((1.0 - self.g * self.g) * (1.0 + mu * mu)) /
                              ((2.0 + self.g * self.g) *
                               (1.0 + self.g * self.g - 2.0 * self.g * mu).powf(1.5));

                for _ in 0..num_samples {
                    let sample_position = r.at(t_current + 0.5 * segment_length);
                    let height = sample_position.to_vector().norm() - self.radius_earth;
                    // compute optical depth for light
                    let h_r = (-height / self.hr).exp() * segment_length;
                    let h_m = (-height / self.hm).exp() * segment_length;
                    optical_depth_r += h_r;
                    optical_depth_m += h_m;
                    // light optical depth
                    let light_ray = Ray::new(sample_position, self.sun_direction);
                    let res = self.atmosphere.intersect_sphere(&light_ray);
                    if res.is_none() {
                        break;
                    }

                    let (_, tl1) = res.unwrap();
                    let segment_length_light = tl1 / num_samples_light as f32;
                    let mut t_current_light = 0.0;
                    let mut optical_depth_light_r = 0.0;
                    let mut optical_depth_light_m = 0.0;
                    let mut exit_early = false;
                    for _ in 0..num_samples_light {
                        let sample_position_light =
                            light_ray.at(t_current_light + 0.5 * segment_length_light);
                        let height_light = sample_position_light.to_vector().norm() -
                                           self.radius_earth;
                        if height_light < 0.0 {
                            exit_early = true;
                            break;
                        }
                        optical_depth_light_r += (-height_light / self.hr).exp() *
                                                 segment_length_light;
                        optical_depth_light_m += (-height_light / self.hm).exp() *
                                                 segment_length_light;
                        t_current_light += segment_length_light;
                    }
                    if !exit_early {
                        let tau = BETA_R * (optical_depth_r + optical_depth_light_r) +
                                  BETA_M * 1.1 * (optical_depth_m + optical_depth_light_m);
                        let attenuation =
                            Colourf::rgb((-tau.r).exp(), (-tau.g).exp(), (-tau.b).exp());
                        sum_r += attenuation * h_r;
                        sum_m += attenuation * h_m;
                    }
                    t_current += segment_length;
                }

                let c = (sum_r * phase_r * BETA_R + sum_m * phase_m * BETA_M) * self.sun_intensity;
                assert!(!c.has_nan());
                c
            }
        }

    }
}
