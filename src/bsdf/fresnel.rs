use std::mem;
use std::fmt::Debug;
use std::f32;

use {Point2f, Vector3f, ONE_MINUS_EPSILON};
use bsdf::{BxDF, BxDFType, MicrofacetDistribution};
use geometry::*;
use material::TransportMode;
use sampling::cosine_sample_hemisphere;
use spectrum::Spectrum;
use clamp;

/// Compute the reflection direction
pub fn reflect(wo: &Vector3f, n: &Vector3f) -> Vector3f {
    -(*wo) + *n * 2.0 * wo.dot(n)
}

/// Compute the refraction direction
pub fn refract(i: &Vector3f, n: &Vector3f, eta: f32) -> Option<Vector3f> {
    let cos_theta_i = n.dot(i);
    let sin2theta_i = (1.0 - cos_theta_i * cos_theta_i).max(0.0);
    let sin2theta_t = eta * eta * sin2theta_i;

    if sin2theta_t >= 1.0 {
        None
    } else {
        let cos_theta_t = (1.0 - sin2theta_t).sqrt();
        Some(eta * -*i + (eta * cos_theta_i - cos_theta_t) * *n)
    }
}

/// Compute the Fresnel coefficient for dielectric materials
pub fn fr_dielectric(cos_theta_i: f32, eta_i: f32, eta_t: f32) -> f32 {
    let mut cos_theta_i = clamp(cos_theta_i, -1.0, 1.0);
    let (mut eta_i, mut eta_t) = (eta_i, eta_t);
    if cos_theta_i <= 0.0 {
        // If leaving the surface, swap the indices of refraction
        mem::swap(&mut eta_i, &mut eta_t);
        cos_theta_i = cos_theta_i.abs();
    }

    let sin_theta_i = (1.0 - cos_theta_i * cos_theta_i).max(0.0).sqrt();
    let sin_theta_t = eta_i / eta_t * sin_theta_i;
    if sin_theta_t >= 1.0 {
        // Total internal reflection
        1.0
    } else {
        let cos_theta_t = (1.0 - sin_theta_t * sin_theta_t).max(0.0).sqrt();
        // Reflectance for parallel polarized light
        let r_parl = ((eta_t * cos_theta_i) - (eta_i * cos_theta_t))
            / ((eta_t * cos_theta_i) + (eta_i * cos_theta_t));
        // Reflectance for perpendicular polarized light
        let r_perp = ((eta_i * cos_theta_i) - (eta_t * cos_theta_t))
            / ((eta_i * cos_theta_i) + (eta_t * cos_theta_t));
        // Total reflectance for unpolarized light
        0.5 * (r_parl * r_parl + r_perp * r_perp)
    }
}

fn fr_conductor(cos_theta_i: f32, eta_i: &Spectrum, eta_t: &Spectrum, k: &Spectrum) -> Spectrum {
    let cos_theta_i = clamp(cos_theta_i, -1.0, 1.0);
    let eta = *eta_t / *eta_i;
    let eta_k = *k / *eta_i;

    let cos2_theta_i = cos_theta_i * cos_theta_i;
    let sin2_theta_i = 1.0 - cos2_theta_i;
    let eta2 = eta * eta;
    let eta_k2 = eta_k * eta_k;

    let t0 = eta2 - eta_k2 - sin2_theta_i;
    let a2plusb2 = (t0 * t0 + 4.0 * eta2 * eta_k2).sqrt();
    let t1 = a2plusb2 + cos2_theta_i;
    let a = (0.5 * (a2plusb2 + t0)).sqrt();
    let t2 = 2.0 * cos_theta_i * a;
    let r_s = (t1 - t2) / (t1 + t2);

    let t3 = cos2_theta_i * a2plusb2 + sin2_theta_i * sin2_theta_i;
    let t4 = t2 * sin2_theta_i;
    let r_p = r_s * (t3 - t4) / (t3 + t4);

    0.5 * (r_p + r_s)
}

/// Trait for Fresnel materials
pub trait Fresnel: Debug {
    fn evaluate(&self, cos_theta_i: f32) -> Spectrum;
}

impl Fresnel {
    pub fn conductor(eta_i: Spectrum, eta_t: Spectrum, k: Spectrum) -> FresnelConductor {
        FresnelConductor {
            eta_i: eta_i,
            eta_t: eta_t,
            k: k,
        }
    }

    pub fn dielectric(eta_i: f32, eta_t: f32) -> FresnelDielectric {
        FresnelDielectric {
            eta_i: eta_i,
            eta_t: eta_t,
        }
    }

    pub fn no_op() -> FresnelNoOp {
        FresnelNoOp {}
    }
}


/// Fresnel for conductor materials
#[derive(Copy, Clone, Debug)]
pub struct FresnelConductor {
    eta_i: Spectrum,
    eta_t: Spectrum,
    k: Spectrum,
}

impl Fresnel for FresnelConductor {
    fn evaluate(&self, cos_theta_i: f32) -> Spectrum {
        fr_conductor(cos_theta_i.abs(), &self.eta_i, &self.eta_t, &self.k)
    }
}

/// Fresnel for dielectric materials
#[derive(Copy, Clone, Debug)]
pub struct FresnelDielectric {
    eta_i: f32,
    eta_t: f32,
}

impl Fresnel for FresnelDielectric {
    fn evaluate(&self, cos_theta_i: f32) -> Spectrum {
        Spectrum::grey(fr_dielectric(cos_theta_i.abs(), self.eta_i, self.eta_t))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FresnelNoOp;

impl Fresnel for FresnelNoOp {
    fn evaluate(&self, _cos_theta_i: f32) -> Spectrum {
        Spectrum::white()
    }
}

/// BRDF for perfect specular reflection
#[derive(Copy, Clone, Debug)]
pub struct SpecularReflection<'a> {
    r: Spectrum,
    fresnel: &'a Fresnel,
}

impl<'a> SpecularReflection<'a> {
    pub fn new(r: Spectrum, fresnel: &'a Fresnel) -> SpecularReflection<'a> {
        SpecularReflection {
            r: r,
            fresnel: fresnel,
        }
    }
}

impl<'a> BxDF for SpecularReflection<'a> {
    fn f(&self, _wo: &Vector3f, _wi: &Vector3f) -> Spectrum {
        // The probability to call f() with the exact (wo, wi) for specular reflection is 0, so we
        // return black here. Use sample_f() instead.
        Spectrum::black()
    }

    fn sample_f(&self, wo: &Vector3f, _sample: &Point2f) -> (Spectrum, Vector3f, f32, BxDFType) {
        // There's only one possible wi for a given wo, so we always return it with a pdf of 1.
        let wi = Vector3f::new(-wo.x, -wo.y, wo.z);
        let spectrum = self.fresnel.evaluate(cos_theta(&wi)) * self.r / abs_cos_theta(&wi);
        (spectrum, wi, 1.0, BxDFType::empty())
    }

    fn pdf(&self, _wo: &Vector3f, _wi: &Vector3f) -> f32 {
        0.0
    }

    fn get_type(&self) -> BxDFType {
        BxDFType::BSDF_SPECULAR | BxDFType::BSDF_REFLECTION
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SpecularTransmission {
    t: Spectrum,
    eta_a: f32,
    eta_b: f32,
    fresnel: FresnelDielectric,
    mode: TransportMode,
}

impl SpecularTransmission {
    pub fn new(t: Spectrum, eta_a: f32, eta_b: f32, mode: TransportMode) -> SpecularTransmission {
        SpecularTransmission {
            t,
            eta_a,
            eta_b,
            fresnel: Fresnel::dielectric(eta_a, eta_b),
            mode,
        }
    }
}

impl BxDF for SpecularTransmission {
    fn f(&self, _wo: &Vector3f, _wi: &Vector3f) -> Spectrum {
        // The probability to call f() with the exact (wo, wi) for specular transmission is 0, so we
        // return black here. Use sample_f() instead.
        Spectrum::black()
    }

    fn sample_f(&self, wo: &Vector3f, _sample: &Point2f) -> (Spectrum, Vector3f, f32, BxDFType) {
        // Figure out which $\eta$ is incident and which is transmitted
        let entering = cos_theta(wo) > 0.0;
        let eta_i = if entering { self.eta_a } else { self.eta_b };
        let eta_t = if entering { self.eta_b } else { self.eta_a };

        // Compute ray direction for specular transmission
        if let Some(wi) = refract(
            wo,
            &face_forward(&Vector3f::new(0.0, 0.0, 1.0), wo),
            eta_i / eta_t,
        ) {
            let mut ft = self.t * (Spectrum::white() - self.fresnel.evaluate(cos_theta(&wi)));

            // Account for non-symmetry with transmission to different medium
            if self.mode == TransportMode::RADIANCE {
                ft = ft * (eta_i * eta_i) / (eta_t * eta_t);
            }

            return (
                ft / abs_cos_theta(&wi),
                wi,
                1.0,
                BxDFType::BSDF_SPECULAR | BxDFType::BSDF_TRANSMISSION,
            );
        } else {
            return (
                Spectrum::white(),
                Vector3f::new(0.0, 0.0, 0.0),
                0.0,
                BxDFType::empty(),
            );
        }
    }

    fn pdf(&self, _wo: &Vector3f, _wi: &Vector3f) -> f32 {
        0.0
    }

    fn get_type(&self) -> BxDFType {
        BxDFType::BSDF_SPECULAR | BxDFType::BSDF_TRANSMISSION
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FresnelSpecular {
    r: Spectrum,
    t: Spectrum,
    eta_a: f32,
    eta_b: f32,
    mode: TransportMode,
}

impl FresnelSpecular {
    pub fn new(
        r: Spectrum,
        t: Spectrum,
        eta_a: f32,
        eta_b: f32,
        mode: TransportMode,
    ) -> FresnelSpecular {
        FresnelSpecular {
            r,
            t,
            eta_a,
            eta_b,
            mode,
        }
    }
}

impl BxDF for FresnelSpecular {
    fn f(&self, _wo: &Vector3f, _wi: &Vector3f) -> Spectrum {
        // The probability to call f() with the exact (wo, wi) for specular reflection is 0, so we
        // return black here. Use sample_f() instead.
        Spectrum::black()
    }

    fn sample_f(&self, wo: &Vector3f, u: &Point2f) -> (Spectrum, Vector3f, f32, BxDFType) {
        let fr = fr_dielectric(cos_theta(wo), self.eta_a, self.eta_b);
        if u[0] < fr {
            // Compute specular reflection for FresnelSpecular

            // Compute perfect specular reflection direction
            let wi = Vector3f::new(-wo.x, -wo.y, wo.z);

            (
                fr * self.r / abs_cos_theta(&wi),
                wi,
                fr,
                BxDFType::BSDF_SPECULAR | BxDFType::BSDF_REFLECTION,
            )
        } else {
            // Compute specular transmission for FresnelSpecular

            // Figure out which $\eta$ is incident and which is transmitted
            let entering = cos_theta(wo) > 0.0;
            let eta_i = if entering { self.eta_a } else { self.eta_b };
            let eta_t = if entering { self.eta_b } else { self.eta_a };

            // Compute ray direction for specular transmission
            if let Some(wi) = refract(
                wo,
                &face_forward(&Vector3f::new(0.0, 0.0, 1.0), wo),
                eta_i / eta_t,
            ) {
                let mut ft = self.t * (1.0 - fr);

                // Account for non-symmetry with transmission to different medium
                if self.mode == TransportMode::RADIANCE {
                    ft *= (eta_i * eta_i) / (eta_t * eta_t);
                }
                (
                    ft / abs_cos_theta(&wi),
                    wi,
                    1.0 - fr,
                    BxDFType::BSDF_SPECULAR | BxDFType::BSDF_TRANSMISSION,
                )
            } else {
                (
                    Spectrum::black(),
                    Vector3f::new(0.0, 0.0, 0.0),
                    0.0,
                    BxDFType::empty(),
                )
            }
        }
    }

    fn pdf(&self, _wo: &Vector3f, _wi: &Vector3f) -> f32 {
        0.0
    }

    fn get_type(&self) -> BxDFType {
        BxDFType::BSDF_SPECULAR | BxDFType::BSDF_REFLECTION | BxDFType::BSDF_TRANSMISSION
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FresnelBlend<'a> {
    rd: Spectrum,
    rs: Spectrum,
    distrib: &'a MicrofacetDistribution,
}

impl<'a> FresnelBlend<'a> {
    pub fn new(rs: Spectrum, rd: Spectrum, distrib: &MicrofacetDistribution) -> FresnelBlend {
        FresnelBlend { rd, rs, distrib }
    }

    fn schlick_fresnel(&self, cos_theta: f32) -> Spectrum {
        self.rs + pow5(1.0 - cos_theta) * (Spectrum::white() - self.rs)
    }
}

impl<'a> BxDF for FresnelBlend<'a> {
    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum {
        let diffuse = (28.0 / (23.0 * f32::consts::PI)) * self.rd * (Spectrum::white() - self.rs)
            * (1.0 - pow5(1.0 - 0.5 * abs_cos_theta(wi)))
            * (1.0 - pow5(1.0 - 0.5 * abs_cos_theta(wo)));

        let mut wh = *wi + *wo;
        if wh.x == 0.0 && wh.y == 0.0 && wh.z == 0.0 {
            return 0.0.into();
        }
        wh = wh.normalize();
        let specular = self.distrib.d(&wh)
            / (4.0 * wi.dot(&wh).abs() * f32::max(abs_cos_theta(wi), abs_cos_theta(wo)))
            * self.schlick_fresnel(wi.dot(&wh));

        diffuse + specular
    }

    fn pdf(&self, wo: &Vector3f, wi: &Vector3f) -> f32 {
        if !same_hemisphere(wo, wi) {
            return 0.0;
        }
        let wh = (*wo + *wi).normalize();
        let pdf_wh = self.distrib.pdf(wo, &wh);

        0.5 * (abs_cos_theta(wi) * f32::consts::FRAC_1_PI + pdf_wh / (4.0 * wo.dot(&wh)))
    }

    fn sample_f(&self, wo: &Vector3f, u_orig: &Point2f) -> (Spectrum, Vector3f, f32, BxDFType) {
        let mut u = *u_orig;
        let mut wi;
        if u[0] < 0.5 {
            u[0] = f32::min(2.0 * u[0], ONE_MINUS_EPSILON);
            // Cosine-sample the hemisphere, flipping the direction if necessary
            wi = cosine_sample_hemisphere(&u);
            if wo.z < 0.0 {
                wi.z *= -1.0;
            }
        } else {
            u[0] = f32::min(2.0 * (u[0] - 0.5), ONE_MINUS_EPSILON);
            // Sample microfacet orientation `wh` and reflected direction `wi`
            let wh = self.distrib.sample_wh(wo, &u);
            wi = reflect(wo, &wh);
            if !same_hemisphere(wo, &wi) {
                return (0.0.into(), wi, 0.0, BxDFType::empty());
            }
        }

        (self.f(wo, &wi), wi, self.pdf(wo, &wi), BxDFType::empty())
    }

    fn get_type(&self) -> BxDFType {
        BxDFType::BSDF_REFLECTION | BxDFType::BSDF_GLOSSY
    }
}

#[inline]
fn pow5(v: f32) -> f32 {
    (v * v) * (v * v) * v
}
