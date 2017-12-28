use std::sync::Arc;
use std::f32;

use light_arena::Allocator;
use num::zero;

use {lerp, clamp, Vector3f, Point2f};
use bsdf::{BSDF, BxDFHolder, BxDF, BxDFType, TrowbridgeReitzDistribution, MicrofacetDistribution,
           SpecularTransmission, MicrofacetReflection, MicrofacetTransmission,
           LambertianTransmission};
use bsdf::{fr_dielectric, reflect, Fresnel};
use geometry::{spherical_direction, same_hemisphere, abs_cos_theta};
use interaction::SurfaceInteraction;
use material::{Material, TransportMode};
use paramset::TextureParams;
use texture::{TextureSpectrum, TextureFloat};
use spectrum::Spectrum;


#[derive(Debug)]
pub struct DisneyMaterial {
    color: Arc<TextureSpectrum>,
    // base_color: Arc<TextureFloat>,
    metallic: Arc<TextureFloat>,
    eta: Arc<TextureFloat>,
    roughness: Arc<TextureFloat>,
    specular_tint: Arc<TextureFloat>,
    anisotropic: Arc<TextureFloat>,
    sheen: Arc<TextureFloat>,
    sheen_tint: Arc<TextureFloat>,
    clearcoat: Arc<TextureFloat>,
    clearcoat_gloss: Arc<TextureFloat>,
    spec_trans: Arc<TextureFloat>,
    scatter_distance: Arc<TextureSpectrum>,
    flatness: Arc<TextureFloat>,
    diff_trans: Arc<TextureFloat>,
    bumpmap: Option<Arc<TextureFloat>>,
    thin: bool,
}

impl DisneyMaterial {
    pub fn create(mp: &mut TextureParams) -> Arc<Material> {
        let color = mp.get_spectrum_texture("color", &Spectrum::from(0.5));
        let metallic = mp.get_float_texture("metallic", 0.0);
        let eta = mp.get_float_texture("eta", 1.5);
        let roughness = mp.get_float_texture("roughness", 0.5);
        let specular_tint = mp.get_float_texture("speculartint", 0.0);
        let anisotropic = mp.get_float_texture("anisotropic", 0.0);
        let sheen = mp.get_float_texture("sheen", 0.0);
        let sheen_tint = mp.get_float_texture("sheentint", 0.5);
        let clearcoat = mp.get_float_texture("clearcoat", 0.0);
        let clearcoat_gloss = mp.get_float_texture("clearcoatgloss", 1.0);
        let spec_trans = mp.get_float_texture("spectrans", 0.0);
        let scatter_distance = mp.get_spectrum_texture("scatterdistance", &Spectrum::from(0.0));
        let thin = mp.find_bool("thin", false);
        let flatness = mp.get_float_texture("flatness", 0.0);
        let diff_trans = mp.get_float_texture("difftrans", 1.0);
        let bumpmap = mp.get_float_texture_or_none("bumpmap");

        Arc::new(DisneyMaterial {
                     color,
                     metallic,
                     eta,
                     roughness,
                     specular_tint,
                     anisotropic,
                     sheen,
                     sheen_tint,
                     clearcoat,
                     clearcoat_gloss,
                     spec_trans,
                     scatter_distance,
                     flatness,
                     diff_trans,
                     bumpmap,
                     thin,
                 })
    }
}

impl Material for DisneyMaterial {
    fn compute_scattering_functions<'a, 'b>(&self,
                                            si: &mut SurfaceInteraction<'a, 'b>,
                                            mode: TransportMode,
                                            _allow_multiple_lobes: bool,
                                            arena: &'b Allocator) {
        if let Some(ref bump) = self.bumpmap {
            super::bump(bump, si);
        }

        let mut bxdfs = BxDFHolder::new(arena);

        // Diffuse
        let c = self.color.evaluate(si).clamp();
        let metallic_weight = self.metallic.evaluate(si);
        let e = self.eta.evaluate(si);
        let strans = self.spec_trans.evaluate(si);
        let diffuse_weight = (1.0 - metallic_weight) * (1.0 - strans);
        let dt = self.diff_trans.evaluate(si) / 2.0; // 0: all diffuse is reflected -> 1, transmitted
        let rough = self.roughness.evaluate(si);
        let lum = c.y();
        // normalize lum. to isolate hue+sat
        let c_tint = if lum > 0.0 {
            c / lum
        } else {
            Spectrum::white()
        };

        let sheen_weight = self.sheen.evaluate(si);
        let c_sheen = if sheen_weight > 0.0 {
            let stint = self.sheen_tint.evaluate(si);
            lerp(stint, Spectrum::white(), c_tint)
        } else {
            Spectrum::black()
        };

        if diffuse_weight > 0.0 {
            if self.thin {
                let flat = self.flatness.evaluate(si);
                // Blend between DisneyDiffuse and fake subsurface based on flatness. Additionally,
                // weight using diff_trans.
                bxdfs.add(arena <- DisneyDiffuse::new(diffuse_weight * (1.0 - flat) * (1.0 - dt) * c));
                bxdfs
                    .add(arena <- DisneyFakeSS::new(diffuse_weight * flat * (1.0 - dt) * c, rough));
            } else {
                let sd = self.scatter_distance.evaluate(si);
                if sd.is_black() {
                    // No subsurface scattering; use regular (Fresnel modified) diffuse.
                    bxdfs.add(arena <- DisneyDiffuse::new(diffuse_weight * c));
                } else {
                    // Use a BSSRDF instead.
                    bxdfs
                        .add(arena <- SpecularTransmission::new(Spectrum::from(1.0), 1.0, e, mode));
                    // TODO: BSSRDF
                }
            }

            // Retro-reflection.
            bxdfs.add(arena <- DisneyRetro::new(diffuse_weight * c, rough));

            // Sheen (if enabled).
            if sheen_weight > 0.0 {
                bxdfs.add(arena <- DisneySheen::new(diffuse_weight * sheen_weight * c_sheen, SheenMode::Reflect));
            }
        }

        // Create the microfacet distribution for metallic and/or specular transmission.
        let aspect = f32::sqrt(1.0 - self.anisotropic.evaluate(si) * 0.9);
        let ax = f32::max(0.001, sqr(rough) / aspect);
        let ay = f32::max(0.001, sqr(rough) * aspect);
        let distrib = arena <- DisneyMicrofacetDistribution::new(ax, ay);

        // Specular is Trowbridge-Reitz with a modified Fresnel function
        let spec_tint = self.specular_tint.evaluate(si);
        let cspec0 = lerp(metallic_weight,
                          schlick_r0_from_eta(e) * lerp(spec_tint, Spectrum::white(), c_tint),
                          c);
        let fresnel = arena <- DisneyFresnel::new(cspec0, metallic_weight, e);
        bxdfs.add(arena <- MicrofacetReflection::new(c, distrib, fresnel));

        // Clearcoat
        let cc = self.clearcoat.evaluate(si);
        if cc > 0.0 {
            bxdfs.add(arena <- DisneyClearCoat::new(cc, self.clearcoat_gloss.evaluate(si)));
        }

        // BTDF
        if strans > 0.0 {
            // Walter et al.'s model, with the provided transmissive term scaled by sqrt(color), so
            // that after two refractionsm we're back to the provided color.
            let t = strans * c.sqrt();
            if self.thin {
                // Scale roughness based on IOR (Burley 2015, Figure 15).
                let rscaled = (0.65 * e - 0.35) * rough;
                let ax = f32::max(0.001, sqr(rscaled) / aspect);
                let ay = f32::max(0.001, sqr(rscaled) * aspect);
                let scaled_distrib = arena <- TrowbridgeReitzDistribution::new(ax, ay);
                bxdfs.add(arena <- MicrofacetTransmission::new(t, scaled_distrib, 1.0, e, mode));
            } else {
                bxdfs.add(arena <- MicrofacetTransmission::new(t, distrib, 1.0, e, mode));
            }
        }

        if self.thin {
            // Lambertian, weighted by (1.0 - diff_trans}
            bxdfs.add(arena <- LambertianTransmission::new(dt * c));
        }

        si.bsdf = Some(Arc::new(BSDF::new(si, 1.0, bxdfs.into_slice())));
    }
}

// DisneyDiffuse
#[derive(Debug, Clone, Copy)]
struct DisneyDiffuse {
    r: Spectrum,
}

impl DisneyDiffuse {
    pub fn new(r: Spectrum) -> DisneyDiffuse {
        DisneyDiffuse { r }
    }
}

impl BxDF for DisneyDiffuse {
    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum {
        let fo = schlick_weight(abs_cos_theta(wo));
        let fi = schlick_weight(abs_cos_theta(wi));

        // Diffuse fresnel - go from 1 at normal incidence to .5 at grazing.
        // Burley 2015, eq (4).
        self.r * f32::consts::FRAC_1_PI * (1.0 - fo / 2.0) * (1.0 - fi / 2.0)
    }

    fn get_type(&self) -> BxDFType {
        BxDFType::BSDF_REFLECTION | BxDFType::BSDF_DIFFUSE
    }
}

// DisneyFakeSS
#[derive(Debug, Clone, Copy)]
struct DisneyFakeSS {
    r: Spectrum,
    roughness: f32,
}

impl DisneyFakeSS {
    pub fn new(r: Spectrum, roughness: f32) -> DisneyFakeSS {
        DisneyFakeSS { r, roughness }
    }
}

impl BxDF for DisneyFakeSS {
    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum {
        let mut wh = *wi + *wo;
        if wh.x == 0.0 && wh.y == 0.0 && wh.z == 0.0 {
            return Spectrum::from(0.0);
        }
        wh = wh.normalize();
        let cos_theta_d = wi.dot(&wh);

        // Fss90 used to "flatten" retroreflection based on roughness
        let fss90 = cos_theta_d * cos_theta_d * self.roughness;
        let fo = schlick_weight(abs_cos_theta(wo));
        let fi = schlick_weight(abs_cos_theta(wi));
        let fss = lerp(fo, 1.0, fss90) * lerp(fi, 1.0, fss90);
        // 1.25 scale is used to (roughly) preserve albedo
        let ss = 1.25 * (fss * (1.0 / (abs_cos_theta(wo) + abs_cos_theta(wi)) - 0.5) + 0.5);

        self.r * f32::consts::FRAC_1_PI * ss
    }

    fn get_type(&self) -> BxDFType {
        BxDFType::BSDF_REFLECTION | BxDFType::BSDF_DIFFUSE
    }
}

// DisneyRetro
#[derive(Debug, Clone, Copy)]
struct DisneyRetro {
    r: Spectrum,
    roughness: f32,
}

impl DisneyRetro {
    pub fn new(r: Spectrum, roughness: f32) -> DisneyRetro {
        DisneyRetro { r, roughness }
    }
}

impl BxDF for DisneyRetro {
    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum {
        let mut wh = *wi + *wo;
        if wh.x == 0.0 && wh.y == 0.0 && wh.z == 0.0 {
            return Spectrum::from(0.0);
        }
        wh = wh.normalize();
        let cos_theta_d = wi.dot(&wh);
        let fo = schlick_weight(abs_cos_theta(wo));
        let fi = schlick_weight(abs_cos_theta(wi));
        let rr = 2.0 * self.roughness * cos_theta_d * cos_theta_d;

        // Burley 2015, eq (4).
        self.r * f32::consts::FRAC_1_PI * rr * (fo + fi + fo * fi * (rr - 1.0))
    }

    fn get_type(&self) -> BxDFType {
        BxDFType::BSDF_REFLECTION | BxDFType::BSDF_DIFFUSE
    }
}

// DisneySheen

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SheenMode {
    Reflect,
    Transmit,
}

#[derive(Debug, Clone, Copy)]
struct DisneySheen {
    r: Spectrum,
    mode: SheenMode,
}

impl DisneySheen {
    pub fn new(r: Spectrum, mode: SheenMode) -> DisneySheen {
        DisneySheen { r, mode }
    }
}

impl BxDF for DisneySheen {
    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum {
        let mut wh = *wi + *wo;
        if wh.x == 0.0 && wh.y == 0.0 && wh.z == 0.0 {
            return Spectrum::from(0.0);
        }
        wh = wh.normalize();
        let cos_theta_d = wi.dot(&wh);

        self.r * schlick_weight(cos_theta_d)
    }

    fn get_type(&self) -> BxDFType {
        BxDFType::BSDF_REFLECTION | BxDFType::BSDF_DIFFUSE
    }
}

// DisneyClearCoat
#[derive(Debug, Clone, Copy)]
struct DisneyClearCoat {
    weight: f32,
    gloss: f32,
}

impl DisneyClearCoat {
    pub fn new(weight: f32, gloss: f32) -> DisneyClearCoat {
        DisneyClearCoat { weight, gloss }
    }
}

impl BxDF for DisneyClearCoat {
    fn f(&self, wo: &Vector3f, wi: &Vector3f) -> Spectrum {
        let mut wh = *wi + *wo;
        if wh.x == 0.0 && wh.y == 0.0 && wh.z == 0.0 {
            return Spectrum::from(0.0);
        }
        wh = wh.normalize();

        // Clearcoat has ior = 1.5 hardcoded -> F0 = 0.04. It then uses the
        // GTR1 distribution, which has even fatter tails than Trowbridge-Reitz
        // (which is GTR2).
        let Dr = GTR1(abs_cos_theta(&wh), lerp(self.gloss, 0.1, 0.001));
        let Fr = fr_schlick(0.04, wo.dot(&wh));
        // The geometric term always based on alpha = 0.25.
        let Gr = smithG_GGX(abs_cos_theta(wo), 0.25) * smithG_GGX(abs_cos_theta(wi), 0.25);

        Spectrum::from(0.25 * self.weight * Gr * Fr * Dr)
    }

    fn sample_f(&self, wo: &Vector3f, u: &Point2f) -> (Spectrum, Vector3f, f32, BxDFType) {
        if wo.z == 0.0 {
            return (Spectrum::black(), zero(), 0.0, self.get_type());
        }

        let alpha = 0.25;
        let alpha2 = alpha * alpha;
        let cos_theta = f32::sqrt(f32::max(0.0,
                                           (1.0 - f32::powf(alpha2, 1.0 - u[0])) / (1.0 - alpha2)));
        let sin_theta = f32::sqrt(f32::max(0.0, 1.0 - cos_theta * cos_theta));
        let phi = 2.0 * f32::consts::PI * u[1];
        let mut wh = spherical_direction(sin_theta, cos_theta, phi);
        if !same_hemisphere(wo, &wh) {
            wh = -wh;
        }
        let wi = reflect(wo, &wh);

        if !same_hemisphere(wo, &wi) {
            return (Spectrum::black(), zero(), 0.0, self.get_type());
        }

        let pdf = self.pdf(wo, &wi);

        (self.f(wo, &wi), wi, pdf, self.get_type())
    }

    fn pdf(&self, wo: &Vector3f, wi: &Vector3f) -> f32 {
        if !same_hemisphere(wo, wi) {
            return 0.0;
        }

        let mut wh = *wo + *wi;
        if wh.x == 0.0 && wh.y == 0.0 && wh.z == 0.0 {
            return 0.0;
        }
        wh = wh.normalize();

        // The sampling routine samples wh exactly from the GTR1 distribution.
        // Thus, the final value of the PDF is just the value of the
        // distribution for wh converted to a mesure with respect to the
        // surface normal.
        let Dr = GTR1(abs_cos_theta(&wh), lerp(self.gloss, 0.1, 0.001));
        Dr / (4.0 * wo.dot(&wh))

    }

    fn get_type(&self) -> BxDFType {
        BxDFType::BSDF_REFLECTION | BxDFType::BSDF_GLOSSY
    }
}

// DisneyFresnel

/// Specialized Fresnel function used for the specular component, based on
/// a mixture between dielectric and the Schlick Fresnel approximation.
#[derive(Debug, Clone, Copy)]
struct DisneyFresnel {
    r0: Spectrum,
    metallic: f32,
    eta: f32,
}

impl DisneyFresnel {
    pub fn new(r0: Spectrum, metallic: f32, eta: f32) -> DisneyFresnel {
        DisneyFresnel { r0, metallic, eta }
    }
}

impl Fresnel for DisneyFresnel {
    fn evaluate(&self, cos_I: f32) -> Spectrum {
        lerp(self.metallic,
             Spectrum::from(fr_dielectric(cos_I, 1.0, self.eta)),
             fr_schlick_spectrum(self.r0, cos_I))
    }
}

#[derive(Debug, Clone, Copy)]
struct DisneyMicrofacetDistribution {
    inner: TrowbridgeReitzDistribution,
}

impl DisneyMicrofacetDistribution {
    fn new(alphax: f32, alphay: f32) -> DisneyMicrofacetDistribution {
        DisneyMicrofacetDistribution { inner: TrowbridgeReitzDistribution::new(alphax, alphay) }
    }
}

impl MicrofacetDistribution for DisneyMicrofacetDistribution {
    fn d(&self, wh: &Vector3f) -> f32 {
        self.inner.d(wh)
    }

    fn lambda(&self, wh: &Vector3f) -> f32 {
        self.inner.lambda(wh)
    }

    fn g(&self, wi: &Vector3f, wo: &Vector3f) -> f32 {
        // Disney uses the separable masking-shadowing model.
        self.g1(wi) * self.g1(wo)
    }

    fn sample_wh(&self, wo: &Vector3f, u: &Point2f) -> Vector3f {
        self.inner.sample_wh(wo, u)
    }

    fn sample_visible_area(&self) -> bool {
        self.inner.sample_visible_area()
    }
}

/// https://seblagarde.wordpress.com/2013/04/29/memo-on-fresnel-equations/
///
/// The Schlick Fresnel approximation is:
///
/// R = R(0) + (1 - R(0)) (1 - cos theta)^5,
///
/// where R(0) is the reflectance at normal indicence.
#[inline]
fn schlick_weight(cos_theta: f32) -> f32 {
    let m = clamp(1.0 - cos_theta, 0.0, 1.0);
    (m * m) * (m * m) * m
}

#[inline]
fn fr_schlick(r0: f32, cos_theta: f32) -> f32 {
    lerp(schlick_weight(cos_theta), r0, 1.0)
}

#[inline]
fn fr_schlick_spectrum(r0: Spectrum, cos_theta: f32) -> Spectrum {
    lerp(schlick_weight(cos_theta), r0, Spectrum::from(1.0))
}

#[inline]
// For a dielectric, R(0) = (eta - 1)^2 / (eta + 1)^2, assuming we're
// coming from air.
fn schlick_r0_from_eta(eta: f32) -> f32 {
    sqr(eta - 1.0) / sqr(eta + 1.0)
}

#[inline]
fn GTR1(cos_theta: f32, alpha: f32) -> f32 {
    let alpha2 = alpha * alpha;

    (alpha2 - 1.0) /
    (f32::consts::PI * f32::log10(alpha2) * (1.0 + (alpha2 - 1.0) * cos_theta * cos_theta))
}

#[inline]
fn smithG_GGX(cos_theta: f32, alpha: f32) -> f32 {
    let alpha2 = alpha * alpha;
    let cos_theta2 = cos_theta * cos_theta;

    1.0 / (cos_theta + f32::sqrt(alpha2 + cos_theta2 - alpha2 * cos_theta2))
}

#[inline]
fn sqr(x: f32) -> f32 {
    x * x
}
