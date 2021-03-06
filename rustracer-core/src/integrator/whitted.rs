use light_arena::Allocator;

use crate::bounds::Bounds2i;
use crate::bsdf;
use crate::integrator::SamplerIntegrator;
use crate::material::TransportMode;
use crate::paramset::ParamSet;
use crate::ray::Ray;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::spectrum::Spectrum;

/// Simple integrator using the original Whitted recursive algorithm. Only handles direct illumination. See
/// ```DirectLightingIntegrator``` for a slighly better integrator that uses better light sampling.
pub struct Whitted {
    pixel_bounds: Bounds2i,
    /// Maximum number of times a ray can bounce before being terminated.
    pub max_ray_depth: u8,
}

impl Whitted {
    pub fn new(n: u8) -> Whitted {
        Whitted {
            max_ray_depth: n,
            pixel_bounds: Bounds2i::new(),
        }
    }

    pub fn create(ps: &ParamSet) -> Box<dyn SamplerIntegrator> {
        let max_depth = ps.find_one_int("maxdepth", 5);
        // TODO pixel_bounds
        Box::new(Self::new(max_depth as u8))
    }
}

impl SamplerIntegrator for Whitted {
    fn pixel_bounds(&self) -> &Bounds2i {
        &self.pixel_bounds
    }

    fn li(
        &self,
        scene: &Scene,
        ray: &mut Ray,
        sampler: &mut dyn Sampler,
        arena: &Allocator<'_>,
        depth: u32,
    ) -> Spectrum {
        let mut colour = Spectrum::black();

        match scene.intersect(ray) {
            Some(mut isect) => {
                let n = isect.shading.n;
                let wo = isect.hit.wo;

                // Compute scattering functions for surface interaction
                isect.compute_scattering_functions(ray, TransportMode::RADIANCE, false, arena);

                // Yuck, there's got to be a better way to do this FIXME
                if isect.bsdf.is_none() {
                    let mut r = isect.spawn_ray(&ray.d);
                    return self.li(scene, &mut r, sampler, arena, depth);
                }
                let bsdf = isect.bsdf.clone().unwrap();

                // Compute emitted light if ray hit an area light source
                colour += isect.le(&wo);

                // Add contribution of each light source
                for light in &scene.lights {
                    let (li, wi, pdf, visibility_tester) =
                        light.sample_li(&isect.hit, sampler.get_2d());
                    if li.is_black() || pdf == 0.0 {
                        continue;
                    }

                    let f = bsdf.f(&wo, &wi, bsdf::BxDFType::all());
                    if !f.is_black() && visibility_tester.unoccluded(scene) {
                        colour += f * li * wi.dotn(&n).abs() / pdf;
                    }
                }

                if depth + 1 < u32::from(self.max_ray_depth) {
                    colour +=
                        self.specular_reflection(ray, &isect, scene, &bsdf, sampler, arena, depth);
                    colour += self
                        .specular_transmission(ray, &isect, scene, &bsdf, sampler, arena, depth);
                }
            }
            None => {
                colour = scene
                    .lights
                    .iter()
                    .fold(Spectrum::black(), |c, l| c + l.le(ray));
            }
        }

        colour
    }
}
