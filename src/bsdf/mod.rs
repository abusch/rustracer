mod bxdf;
mod fresnel;
mod lambertian;
mod oren_nayar;
mod microfacet;

pub use self::bxdf::*;
pub use self::fresnel::*;
pub use self::lambertian::*;
pub use self::oren_nayar::*;
pub use self::microfacet::*;

use std::cmp;

use {Point2f, Vector3f, ONE_MINUS_EPSILON};
use spectrum::Spectrum;
use interaction::SurfaceInteraction;

bitflags! {
    pub struct BxDFType: u32 {
        const BSDF_REFLECTION   = 0b_00000001;
        const BSDF_TRANSMISSION = 0b_00000010;
        const BSDF_DIFFUSE      = 0b_00000100;
        const BSDF_GLOSSY       = 0b_00001000;
        const BSDF_SPECULAR     = 0b_00010000;
    }
}

/// Represents the Bidirectional Scattering Distribution Function.
/// It represents the properties of a material at a given point.
pub struct BSDF<'a> {
    /// Index of refraction of the surface
    pub eta: f32,
    /// Shading normal (i.e. potentially affected by bump-mapping)
    ns: Vector3f,
    /// Geometry normal
    ng: Vector3f,
    ss: Vector3f,
    ts: Vector3f,
    bxdfs: &'a [&'a BxDF],
}

impl<'a> BSDF<'a> {
    pub fn new<'b>(isect: &'b SurfaceInteraction, eta: f32, bxdfs: &'a [&'a BxDF]) -> BSDF<'a> {
        let ss = isect.dpdu.normalize();
        BSDF {
            eta: eta,
            ns: isect.shading.n,
            ng: isect.n,
            ss: ss,
            ts: isect.shading.n.cross(&ss),
            bxdfs: bxdfs,
        }
    }

    /// Evaluate the BSDF for the given incoming light direction and outgoing light direction.
    pub fn f(&self, wo_w: &Vector3f, wi_w: &Vector3f, flags: BxDFType) -> Spectrum {
        let wi = self.world_to_local(wi_w);
        let wo = self.world_to_local(wo_w);
        if wo.z == 0.0 {
            return Spectrum::black();
        }
        let reflect = wi_w.dot(&self.ng) * wo_w.dot(&self.ng) > 0.0;
        self.bxdfs
            .iter()
            .filter(|b| {
                // Make sure we only evaluate reflection or transmission based on whether wi and wo
                // lie in the same hemisphere.
                b.matches(flags)
                    && ((reflect && (b.get_type().contains(BxDFType::BSDF_REFLECTION)))
                        || (!reflect && (b.get_type().contains(BxDFType::BSDF_TRANSMISSION))))
            })
            .fold(Spectrum::black(), |c, b| c + b.f(&wo, &wi))
    }

    pub fn pdf(&self, wo_w: &Vector3f, wi_w: &Vector3f, flags: BxDFType) -> f32 {
        if self.bxdfs.is_empty() {
            return 0.0;
        }
        let wo = self.world_to_local(wo_w);
        if wo.z == 0.0 {
            return 0.0;
        }
        let wi = self.world_to_local(wi_w);

        let mut matched_comps = 0;
        let mut pdf = 0.0;
        for bxdf in self.bxdfs {
            if bxdf.matches(flags) {
                matched_comps += 1;
                pdf += bxdf.pdf(&wo, &wi);
            }
        }
        if matched_comps == 0 {
            0.0
        } else {
            pdf / matched_comps as f32
        }
    }

    pub fn sample_f(
        &self,
        wo_w: &Vector3f,
        u: &Point2f,
        flags: BxDFType,
    ) -> (Spectrum, Vector3f, f32, BxDFType) {
        let matching_comps = self.bxdfs
            .iter()
            .filter(|b| b.matches(flags))
            .collect::<Vec<&&BxDF>>();
        if matching_comps.is_empty() {
            return (
                Spectrum::black(),
                Vector3f::new(0.0, 0.0, 0.0),
                0.0,
                BxDFType::empty(),
            );
        }
        // Chose which BxDF to sample
        let comp = cmp::min(
            (u[0] * matching_comps.len() as f32).floor() as usize,
            matching_comps.len() - 1,
        );
        let bxdf = matching_comps
            .get(comp)
            .expect("Was expecting a BxDF with this index!");

        // debug!(
        //     "BDDF::sample_f chose comp = {} / matching {}, bxdf = {:?}",
        //     comp,
        //     matching_comps.len(),
        //     bxdf
        // );

        // Remap BxDF sample u to [0,1)^2
        let u_remapped = Point2f::new(
            (u[0] * matching_comps.len() as f32 - comp as f32).min(ONE_MINUS_EPSILON),
            u[1],
        );
        // debug!("u_remapped={}", u_remapped);
        // Sample chosen BxDF
        let wo = self.world_to_local(wo_w);
        if wo.z == 0.0 {
            return (
                Spectrum::black(),
                Vector3f::new(0.0, 0.0, 0.0),
                0.0,
                bxdf.get_type(),
            );
        }
        let (mut f, wi, mut pdf, sampled_type) = bxdf.sample_f(&wo, &u_remapped);
        // debug!(
        //     "For wo = {:?}, sampled f = {}, pdf = {}, ratio = {}, wi = {:?}",
        //     wo,
        //     f,
        //     pdf,
        //     if pdf > 0.0 {
        //         f / pdf
        //     } else {
        //         Spectrum::black()
        //     },
        //     wi
        // );
        if pdf == 0.0 {
            return (
                Spectrum::black(),
                Vector3f::new(0.0, 0.0, 0.0),
                0.0,
                BxDFType::empty(),
            );
        }
        let wi_w = self.local_to_world(&wi);

        // Compute overall PDF with all matching BxDF
        if !bxdf.get_type().contains(BxDFType::BSDF_SPECULAR) && matching_comps.len() > 1 {
            for i in 0..matching_comps.len() {
                if i != comp {
                    pdf += matching_comps[i].pdf(&wo, &wi);
                }
            }
        }
        if matching_comps.len() > 1 {
            pdf /= matching_comps.len() as f32;
        }

        // Compute value of BSDF for sampled direction
        if !bxdf.get_type().contains(BxDFType::BSDF_SPECULAR) && matching_comps.len() > 1 {
            let reflect = wi_w.dot(&self.ng) * wo_w.dot(&self.ng) > 0.0;
            f = matching_comps
                .iter()
                .filter(|b| {
                    (reflect && b.get_type().contains(BxDFType::BSDF_REFLECTION))
                        || (!reflect && b.get_type().contains(BxDFType::BSDF_TRANSMISSION))
                })
                .fold(Spectrum::black(), |f, b| f + b.f(&wo, &wi));
        }

        // debug!(
        //     "Overall f = {}, pdf = {}, ratio = {}",
        //     f,
        //     pdf,
        //     if pdf > 0.0 {
        //         f / pdf
        //     } else {
        //         Spectrum::black()
        //     }
        // );
        (f, wi_w, pdf, sampled_type)
    }

    fn world_to_local(&self, v: &Vector3f) -> Vector3f {
        Vector3f::new(v.dot(&self.ss), v.dot(&self.ts), v.dot(&self.ns))
    }

    fn local_to_world(&self, v: &Vector3f) -> Vector3f {
        Vector3f::new(
            self.ss.x * v.x + self.ts.x * v.y + self.ns.x * v.z,
            self.ss.y * v.x + self.ts.y * v.y + self.ns.y * v.z,
            self.ss.z * v.x + self.ts.z * v.y + self.ns.z * v.z,
        )
    }

    /// Return the number of BxDFs matching the given flags
    pub fn num_components(&self, flags: BxDFType) -> usize {
        self.bxdfs.iter().filter(|b| b.matches(flags)).count()
    }
}

#[test]
fn test_flags() {
    let flags = BxDFType::BSDF_SPECULAR | BxDFType::BSDF_REFLECTION;
    let bxdf_type =
        BxDFType::BSDF_SPECULAR | BxDFType::BSDF_REFLECTION | BxDFType::BSDF_TRANSMISSION;

    assert!((bxdf_type & flags) == flags);
}
