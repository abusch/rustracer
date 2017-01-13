use std::f32::consts;

use na::{Norm, Dot};

use ::{Vector3f, Point2f};
use spectrum::Spectrum;
use bsdf::{BxDF, BxDFType, BSDF_REFLECTION, BSDF_GLOSSY, reflect};
use geometry::{tan_theta, tan2_theta, cos_theta, abs_cos_theta, cos2_theta, cos_phi, cos2_phi,
               sin_phi, sin2_phi, same_hemisphere, spherical_direction, erf, erf_inv};
use bsdf::fresnel::Fresnel;

pub struct MicrofacetReflection {
    r: Spectrum,
    distribution: Box<MicrofacetDistribution + Send + Sync>,
    fresnel: Box<Fresnel + Send + Sync>,
}

impl MicrofacetReflection {
    pub fn new(r: Spectrum,
               distribution: Box<MicrofacetDistribution + Send + Sync>,
               fresnel: Box<Fresnel + Send + Sync>)
               -> MicrofacetReflection {
        MicrofacetReflection {
            r: r,
            distribution: distribution,
            fresnel: fresnel,
        }
    }
}

impl BxDF for MicrofacetReflection {
    fn f(&self, wi: &Vector3f, wo: &Vector3f) -> Spectrum {
        let cos_theta_o = abs_cos_theta(wo);
        let cos_theta_i = abs_cos_theta(wi);
        let mut wh = *wi + *wo;

        // Handle degenerate case for microfacet reflection
        if cos_theta_o == 0.0 || cos_theta_i == 0.0 {
            return Spectrum::black();
        }
        if wh.x == 0.0 && wh.y == 0.0 && wh.z == 0.0 {
            return Spectrum::black();
        }

        wh = wh.normalize();
        let f = self.fresnel.evaluate(wi.dot(&wh));
        self.r * self.distribution.d(&wh) * self.distribution.g(wo, wi) * f /
        (4.0 * cos_theta_i * cos_theta_o)
    }

    fn get_type(&self) -> BxDFType {
        BSDF_REFLECTION | BSDF_GLOSSY
    }

    /// Override sample_f() to use a better importance sampling method than weighted cosine based
    /// on the microface distribution
    fn sample_f(&self, wo: &Vector3f, u: &Point2f) -> (Spectrum, Vector3f, f32, BxDFType) {
        if wo.z == 0.0 {
            return (Spectrum::black(), Vector3f::new(0.0, 0.0, 0.0), 0.0, BxDFType::empty());
        }

        let wh = self.distribution.sample_wh(wo, u);
        let wi = reflect(wo, &wh);
        if !same_hemisphere(wo, &wi) {
            return (Spectrum::black(), Vector3f::new(0.0, 0.0, 0.0), 0.0, BxDFType::empty());
        }
        let pdf = self.distribution.pdf(wo, &wh) / (4.0 * wo.dot(&wh));

        (self.f(wo, &wi), wi, pdf, BxDFType::empty())
    }
}

// TODO MicrofacetTransmission

// Microfacet distributions
pub trait MicrofacetDistribution {
    fn d(&self, wh: &Vector3f) -> f32;

    fn lambda(&self, wh: &Vector3f) -> f32;

    fn g1(&self, wh: &Vector3f) -> f32 {
        1.0 / (1.0 + self.lambda(wh))
    }

    fn g(&self, wi: &Vector3f, wo: &Vector3f) -> f32 {
        1.0 / (1.0 + self.lambda(wi) + self.lambda(wo))
    }

    fn pdf(&self, wo: &Vector3f, wh: &Vector3f) -> f32 {
        if self.sample_visible_area() {
            self.d(wh) * self.g1(wo) * wo.dot(wh).abs() / abs_cos_theta(wo)
        } else {
            self.d(wh) * abs_cos_theta(wh)
        }
    }

    fn sample_wh(&self, wo: &Vector3f, u: &Point2f) -> Vector3f;

    fn sample_visible_area(&self) -> bool;
}

pub struct BeckmannDistribution {
    alpha_x: f32,
    alpha_y: f32,
    sample_visible_area: bool,
}

impl BeckmannDistribution {
    pub fn new(ax: f32, ay: f32) -> BeckmannDistribution {
        BeckmannDistribution {
            alpha_x: ax,
            alpha_y: ay,
            sample_visible_area: true,
        }
    }

    fn sample(&self, wi: &Vector3f, u1: f32, u2: f32) -> Vector3f {
        // 1. stretch wi
        let wi_stretched = Vector3f::new(self.alpha_x * wi.x, self.alpha_y * wi.y, wi.z)
            .normalize();

        // 2. simulate P22_{wi}(x_slope, y_slope, 1, 1)
        let (mut slope_x, mut slope_y) = self.sample11(cos_theta(&wi_stretched), u1, u2);

        // 3. rotate
        let tmp = cos_phi(&wi_stretched) * slope_x - sin_phi(&wi_stretched) * slope_y;
        slope_y = sin_phi(&wi_stretched) * slope_x + cos_phi(&wi_stretched) * slope_y;
        slope_x = tmp;

        // 4. unstretch
        slope_x = self.alpha_x * slope_x;
        slope_y = self.alpha_y * slope_y;

        // 5. compute normal
        Vector3f::new(-slope_x, -slope_y, 1.0).normalize()
    }

    fn sample11(&self, cos_theta_i: f32, u1: f32, u2: f32) -> (f32, f32) {
        // Special case (normal incidence)
        if cos_theta_i > 0.9999 {
            let r = (-(1.0 - u1).ln()).sqrt();
            let sin_phi = (2.0 * consts::PI * u2).sin();
            let cos_phi = (2.0 * consts::PI * u2).cos();
            return (r * cos_phi, r * sin_phi);
        }

        // The original inversion routine from the paper contained
        // discontinuities, which causes issues for QMC integration
        // and techniques like Kelemen-style MLT. The following code
        // performs a numerical inversion with better behavior
        let sin_theta_i = (1.0 - cos_theta_i * cos_theta_i).max(0.0).sqrt();
        let tan_theta_i = sin_theta_i / cos_theta_i;
        let cot_theta_i = 1.0 / tan_theta_i;

        // Search interval -- everything is parameterized
        // in the Erf() domain
        let mut a = -1.0;
        let mut c = erf(cot_theta_i);
        let sample_x = u1.max(1e-6);

        // Start with a good initial guess
        // Float b = (1-sample_x) * a + sample_x * c;

        // We can do better (inverse of an approximation computed in
        // Mathematica)
        let theta_i = cos_theta_i.acos();
        let fit = 1.0 + theta_i * (-0.876 + theta_i * (0.4265 - 0.0594 * theta_i));
        let mut b = c - (1.0 + c) * (1.0 - sample_x).powf(fit);

        // Normalization factor for the CDF
        const SQRT_PI_INV: f32 = consts::FRAC_2_SQRT_PI * 0.5;
        let normalization =
            1.0 / (1.0 + c + SQRT_PI_INV * tan_theta_i * (-cot_theta_i * cot_theta_i).exp());

        let mut it = 0;
        loop {
            it += 1;
            if it >= 10 {
                break;
            }

            // Bisection criterion -- the oddly-looking
            // Boolean expression are intentional to check
            // for NaNs at little additional cost
            if !(b >= a && b <= c) {
                b = 0.5 * (a + c);
            }

            // Evaluate the CDF and its derivative
            // (i.e. the density function)
            let inv_erf = erf_inv(b);
            let value = normalization *
                        (1.0 + b + SQRT_PI_INV * tan_theta_i * (-inv_erf * inv_erf).exp()) -
                        sample_x;
            let derivative = normalization * (1.0 - inv_erf * tan_theta_i);

            if value.abs() < 1e-5 {
                break;
            }

            // Update bisection intervals
            if value > 0.0 {
                c = b;
            } else {
                a = b;
            }

            b -= value / derivative;
        }

        // Now convert back into a slope value
        let slope_x = erf_inv(b);

        // Simulate Y component
        let slope_y = erf_inv(2.0 * u2.max(1e-6) - 1.0);

        assert!(!slope_x.is_infinite());
        assert!(!slope_x.is_nan());
        assert!(!slope_y.is_infinite());
        assert!(!slope_y.is_nan());

        (slope_x, slope_y)
    }
}

impl MicrofacetDistribution for BeckmannDistribution {
    fn d(&self, wh: &Vector3f) -> f32 {
        let tan2theta = tan2_theta(wh);
        if tan2theta.is_infinite() {
            return 0.0;
        }

        let cos4_theta = cos2_theta(wh) * cos2_theta(wh);
        (-tan2theta *
         (cos2_phi(wh) / (self.alpha_x * self.alpha_x) +
          sin2_phi(wh) / (self.alpha_y * self.alpha_y)))
            .exp() / (consts::PI * self.alpha_x * self.alpha_y * cos4_theta)
    }

    fn lambda(&self, wh: &Vector3f) -> f32 {
        let abs_tan_theta = tan_theta(wh).abs();
        if abs_tan_theta.is_infinite() {
            return 0.0;
        }

        // Compute alpha for direction w
        let alpha = (cos_phi(wh) * self.alpha_x * self.alpha_x +
                     sin_phi(wh) * self.alpha_y * self.alpha_y)
            .sqrt();

        let a = 1.0 / (alpha * abs_tan_theta);
        if a >= 1.6 {
            0.0
        } else {
            (1.0 - 1.259 * a + 0.396 * a * a) / (3.535 * a + 2.181 * a * a)
        }
    }

    fn sample_wh(&self, wo: &Vector3f, u: &Point2f) -> Vector3f {
        if !self.sample_visible_area {
            // Sample full distribution of normals
            let (tan_2_theta, phi) = if self.alpha_x == self.alpha_y {
                let mut log_sample = u[0].ln();
                if log_sample.is_infinite() {
                    log_sample = 0.0;
                }
                (-self.alpha_x * self.alpha_x * log_sample, u[1] * 2.0 * consts::PI)
            } else {
                // Compute tan_2_theta and phi for anisotropic Beckmann distribution
                let mut log_sample = u[0].ln();
                if log_sample.is_infinite() {
                    log_sample = 0.0;
                }
                let mut phi = (self.alpha_y / self.alpha_x *
                               (2.0 * consts::PI * u[1] + consts::FRAC_PI_2).tan())
                    .atan();
                if u[1] > 0.5 {
                    phi += consts::PI;
                }
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();
                let alpha_x_2 = self.alpha_x * self.alpha_x;
                let alpha_y_2 = self.alpha_y * self.alpha_y;
                let tan2_theta = -log_sample /
                                 (cos_phi * cos_phi / alpha_x_2 + sin_phi * sin_phi / alpha_y_2);
                (tan2_theta, phi)
            };
            let cos_theta = 1.0 / (1.0 + tan_2_theta).sqrt();
            let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
            let mut wh = spherical_direction(sin_theta, cos_theta, phi);
            if !same_hemisphere(wo, &wh) {
                wh = -wh;
            }
            wh
        } else {
            // Sample visible area of normals
            let flip = wo.z < 0.0;
            let wo = if flip { -(*wo) } else { *wo };
            let mut wh = self.sample(&wo, u[0], u[1]);
            if flip { -wh } else { wh }
        }
    }

    fn sample_visible_area(&self) -> bool {
        self.sample_visible_area
    }
}

pub struct TrowbridgeReitzDistribution {
    alpha_x: f32,
    alpha_y: f32,
    sample_visible_area: bool,
}

impl TrowbridgeReitzDistribution {
    pub fn new(ax: f32, ay: f32) -> TrowbridgeReitzDistribution {
        TrowbridgeReitzDistribution {
            alpha_x: ax,
            alpha_y: ay,
            sample_visible_area: true,
        }
    }

    pub fn roughness_to_alpha(roughness: f32) -> f32 {
        let roughness = roughness.max(1e-3);
        let x = roughness.ln();
        1.62142 + 0.819955 * x + 0.1734 * x * x + 0.0171201 * x * x * x +
        0.000640711 * x * x * x * x

    }

    fn sample(&self, wi: &Vector3f, u1: f32, u2: f32) -> Vector3f {
        // 1. stretch wi
        let wi_stretched = Vector3f::new(self.alpha_x * wi.x, self.alpha_y * wi.y, wi.z)
            .normalize();

        // 2. simulate P22_{wi}(x_slope, y_slope, 1, 1)
        let (mut slope_x, mut slope_y) = self.sample11(cos_theta(&wi_stretched), u1, u2);

        // 3. rotate
        let tmp = cos_phi(&wi_stretched) * slope_x - sin_phi(&wi_stretched) * slope_y;
        slope_y = sin_phi(&wi_stretched) * slope_x + cos_phi(&wi_stretched) * slope_y;
        slope_x = tmp;

        // 4. unstretch
        slope_x = self.alpha_x * slope_x;
        slope_y = self.alpha_y * slope_y;

        // 5. compute normal
        Vector3f::new(-slope_x, -slope_y, 1.0).normalize()
    }

    fn sample11(&self, cos_theta: f32, u1: f32, u2: f32) -> (f32, f32) {
        // special case (normal incidence)
        if cos_theta > 0.9999 {
            let r = (u1 / (1.0 - u1)).sqrt();
            let phi = 6.28318530718 * u2;
            return (r * phi.cos(), r * phi.sin());
        }

        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
        let tan_theta = sin_theta / cos_theta;
        let a = 1.0 / tan_theta;
        let G1 = 2.0 / (1.0 + (1.0 + 1.0 / (a * a)).sqrt());

        // sample slope_x
        let A = 2.0 * u1 / G1 - 1.0;
        let mut tmp = 1.0 / (A * A - 1.0);
        if tmp > 1e10 {
            tmp = 1e10;
        }
        let B = tan_theta;
        let D = (B * B * tmp * tmp - (A * A - B * B) * tmp).max(0.0).sqrt();
        let slope_x_1 = B * tmp - D;
        let slope_x_2 = B * tmp + D;
        let slope_x = if A < 0.0 || slope_x_2 > 1.0 / tan_theta {
            slope_x_1
        } else {
            slope_x_2
        };

        // sample slope_y
        let (S, u2) = if u2 > 0.5 {
            (1.0, 2.0 * (u2 - 0.5))
        } else {
            (-1.0, 2.0 * (0.5 - u2))
        };
        let z = (u2 * (u2 * (u2 * 0.27385 - 0.73369) + 0.46341)) /
                (u2 * (u2 * (u2 * 0.093073 + 0.309420) - 1.000000) + 0.597999);
        let slope_y = S * z * (1.0 + slope_x * slope_x).sqrt();

        assert!(!slope_y.is_infinite());
        assert!(!slope_y.is_nan());
        (slope_x, slope_y)
    }
}

impl MicrofacetDistribution for TrowbridgeReitzDistribution {
    fn d(&self, wh: &Vector3f) -> f32 {
        let tan2theta = tan2_theta(wh);
        if tan2theta.is_infinite() {
            return 0.0;
        }

        let cos4theta = cos2_theta(wh) * cos2_theta(wh);
        let e = (cos2_phi(wh) / (self.alpha_x * self.alpha_x) +
                 sin2_phi(wh) / (self.alpha_y * self.alpha_y)) * tan2theta;

        1.0 / (consts::PI * self.alpha_x * self.alpha_y * cos4theta * (1.0 + e) * (1.0 + e))
    }

    fn lambda(&self, wh: &Vector3f) -> f32 {
        let abs_tan_theta = tan_theta(wh).abs();
        if abs_tan_theta.is_infinite() {
            return 0.0;
        }

        // Compute alpha for direction w
        let alpha = (cos2_phi(wh) * self.alpha_x * self.alpha_x +
                     sin2_phi(wh) * self.alpha_y * self.alpha_y)
            .sqrt();
        let alpha2tan2theta = (alpha * abs_tan_theta) * (alpha * abs_tan_theta);
        (-1.0 + (1.0 + alpha2tan2theta).sqrt()) / 2.0
    }

    fn sample_wh(&self, wo: &Vector3f, u: &Point2f) -> Vector3f {
        if !self.sample_visible_area {
            let mut cos_theta = 0.0;
            let phi = (2.0 * consts::PI) * u[1];

            if self.alpha_x == self.alpha_y {
                let tan_theta2 = self.alpha_x * self.alpha_x * u[0] / (1.0 - u[0]);
                cos_theta = 1.0 / (1.0 + tan_theta2).sqrt();
            } else {
                let mut phi = (self.alpha_y / self.alpha_x *
                               (2.0 * consts::PI * u[1] + 0.5 * consts::PI).tan())
                    .atan();
                if u[1] > 0.5 {
                    phi += consts::PI;
                }
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();
                let alpha_x_2: f32 = self.alpha_x * self.alpha_x;
                let alpha_y_2: f32 = self.alpha_y * self.alpha_y;
                let alpha_2: f32 = 1.0 /
                                   (cos_phi * cos_phi / alpha_x_2 + sin_phi * sin_phi / alpha_y_2);
                let tan_theta2 = alpha_2 * u[0] / (1.0 - u[0]);
                cos_theta = 1.0 / (1.0 + tan_theta2).sqrt();
            }
            let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
            let mut wh = spherical_direction(sin_theta, cos_theta, phi);
            if !same_hemisphere(wo, &wh) { -wh } else { wh }
        } else {
            let flip = wo.z < 0.0;
            let wo = if flip { -(*wo) } else { *wo };
            let mut wh = self.sample(&wo, u[0], u[1]);
            if flip { -wh } else { wh }
        }
    }

    fn sample_visible_area(&self) -> bool {
        self.sample_visible_area
    }
}
