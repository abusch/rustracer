use std::f32;

use {Point2f, Vector};
use interaction::{Interaction, SurfaceInteraction};
use ray::Ray;
use scene::Scene;
use spectrum::Spectrum;

mod point;
mod distant;
mod diffuse;

pub use self::point::PointLight;
pub use self::distant::DistantLight;
pub use self::diffuse::DiffuseAreaLight;

bitflags! {
    pub flags LightFlags: u32 {
        const DELTA_POSITION  = 0b_00000001,
        const DELTA_DIRECTION = 0b_00000010,
        const AREA            = 0b_00000100,
        const INFINITE        = 0b_00001000,
    }
}

#[inline]
fn is_delta_light(flags: LightFlags) -> bool {
    flags.contains(DELTA_POSITION) || flags.contains(DELTA_DIRECTION)
}

pub struct VisibilityTester {
    ray: Ray,
}

impl VisibilityTester {
    pub fn new(ray: Ray) -> VisibilityTester {
        VisibilityTester { ray: ray }
    }

    pub fn unoccluded(&self, scene: &Scene) -> bool {
        !scene.intersect_p(&self.ray)
    }
}

pub trait Light {
    /// Sample the light source for an outgoing direction wo.
    /// Return a triplet of:
    ///  * emitted light in the sampled direction
    ///  * the sampled direction wi
    ///  * the pdf for that direction
    ///  * A VisibilityTester
    fn sample_li(&self,
                 isect: &SurfaceInteraction,
                 wo: &Vector,
                 u: &Point2f)
                 -> (Spectrum, Vector, f32, VisibilityTester);

    fn pdf_li(&self, si: &SurfaceInteraction, wi: &Vector) -> f32;

    fn preprocess(&mut self, _scene: &Scene) {}

    fn n_samples(&self) -> u32;

    fn flags(&self) -> LightFlags;

    fn power(&self) -> Spectrum;
}

pub trait AreaLight: Light {
    fn l(&self, si: &Interaction, w: &Vector) -> Spectrum;
}
