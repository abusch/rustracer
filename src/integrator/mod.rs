use na::Dot;

use bsdf;
use spectrum::Spectrum;
use interaction::SurfaceInteraction;
use ray::Ray;
use sampler::Sampler;
use scene::Scene;

mod whitted;
mod ao;
mod normal;

pub use self::whitted::Whitted;
pub use self::ao::AmbientOcclusion;
pub use self::normal::Normal;

pub trait SamplerIntegrator {
    fn li(&self, scene: &Scene, ray: &mut Ray, sampler: &mut Sampler, depth: u32) -> Spectrum;

    fn specular_reflection(&self,
                           ray: &mut Ray,
                           isect: &SurfaceInteraction,
                           scene: &Scene,
                           bsdf: &bsdf::BSDF,
                           sampler: &mut Sampler,
                           depth: u32)
                           -> Spectrum {
        let flags = bsdf::BSDF_REFLECTION | bsdf::BSDF_SPECULAR;
        // TODO use sampler.get_2d()
        let (f, wi, pdf) = bsdf.sample_f(&isect.wo, (0.0, 0.0), flags);
        let ns = isect.shading.n;
        if !f.is_black() && pdf != 0.0 && wi.dot(&ns) != 0.0 {
            let mut r = ray.spawn(isect.p, wi);
            let refl = self.li(scene, &mut r, sampler, depth + 1);
            f * refl * wi.dot(&ns).abs() / pdf
        } else {
            Spectrum::black()
        }
    }

    fn specular_transmission(&self,
                             ray: &mut Ray,
                             isect: &SurfaceInteraction,
                             scene: &Scene,
                             bsdf: &bsdf::BSDF,
                             sampler: &mut Sampler,
                             depth: u32)
                             -> Spectrum {
        let flags = bsdf::BSDF_TRANSMISSION | bsdf::BSDF_SPECULAR;
        // TODO use sampler.get_2d()
        let (f, wi, pdf) = bsdf.sample_f(&isect.wo, (0.0, 0.0), flags);
        let ns = isect.shading.n;
        if !f.is_black() && pdf != 0.0 && wi.dot(&ns) != 0.0 {
            let mut r = ray.spawn(isect.p, wi);
            let refr = self.li(scene, &mut r, sampler, depth + 1);
            f * refr * wi.dot(&ns).abs() / pdf
        } else {
            Spectrum::black()
        }
    }
}
