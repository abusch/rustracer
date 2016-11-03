use na::Dot;

use bsdf;
use colour::Colourf;
use intersection::Intersection;
use ray::Ray;
use sampling::Sampler;
use scene::Scene;

mod whitted;
mod ao;
mod normal;

pub use self::whitted::Whitted;
pub use self::ao::AmbientOcclusion;
pub use self::normal::Normal;

pub trait SamplerIntegrator {
    fn li(&self, scene: &Scene, ray: &mut Ray, sampler: &mut Sampler, depth: u32) -> Colourf;

    fn specular_reflection(&self,
                           ray: &mut Ray,
                           isect: &Intersection,
                           scene: &Scene,
                           sampler: &mut Sampler,
                           depth: u32)
                           -> Colourf {
        let flags = bsdf::REFLECTION | bsdf::SPECULAR;
        // TODO use sampler.get_2d()
        let (f, wi, pdf) = isect.bsdf.sample_f(&isect.wo, (0.0, 0.0), flags);
        let ns = isect.dg.nhit;
        if !f.is_black() && pdf != 0.0 && wi.dot(&ns) != 0.0 {
            let mut r = ray.spawn(isect.dg.phit, wi);
            f * self.li(scene, &mut r, sampler, depth + 1) * wi.dot(&ns).abs() / pdf
        } else {
            Colourf::black()
        }
    }
}
