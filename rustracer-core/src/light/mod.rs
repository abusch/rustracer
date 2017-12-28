use std::f32;
use std::fmt::Debug;

use parking_lot::Mutex;

use {Point2f, Vector3f};
use interaction::Interaction;
use ray::Ray;
use scene::Scene;
use spectrum::Spectrum;

mod point;
mod distant;
mod diffuse;
mod infinite;

pub use self::point::PointLight;
pub use self::distant::DistantLight;
pub use self::diffuse::DiffuseAreaLight;
pub use self::infinite::InfiniteAreaLight;

bitflags! {
    pub struct LightFlags: u32 {
        const DELTA_POSITION  = 0b_0000_0001;
        const DELTA_DIRECTION = 0b_0000_0010;
        const AREA            = 0b_0000_0100;
        const INFINITE        = 0b_0000_1000;
    }
}

static COUNTER: Mutex<u32> = Mutex::new(0);

#[inline]
pub fn is_delta_light(flags: LightFlags) -> bool {
    flags.contains(LightFlags::DELTA_POSITION) || flags.contains(LightFlags::DELTA_DIRECTION)
}

pub struct VisibilityTester {
    pub p0: Interaction,
    pub p1: Interaction,
}

impl VisibilityTester {
    pub fn new(p0: Interaction, p1: Interaction) -> VisibilityTester {
        VisibilityTester { p0: p0, p1: p1 }
    }

    pub fn unoccluded(&self, scene: &Scene) -> bool {
        let r = self.p0.spawn_ray_to_interaction(&self.p1);
        !scene.intersect_p(&r)
    }
}

pub fn get_next_id() -> u32 {
    let mut counter = COUNTER.lock();
    let id = *counter;
    *counter += 1;

    id
}

pub trait Light: Debug + Send + Sync {
    fn id(&self) -> u32;
    /// Sample the light source
    /// Return a tuple of:
    ///  * emitted light in the sampled direction
    ///  * the sampled direction wi
    ///  * the pdf for that direction
    ///  * A VisibilityTester
    fn sample_li(&self,
                 isect: &Interaction,
                 u: &Point2f)
                 -> (Spectrum, Vector3f, f32, VisibilityTester);

    fn pdf_li(&self, si: &Interaction, wi: &Vector3f) -> f32;

    fn preprocess(&self, _scene: &Scene) {}

    fn n_samples(&self) -> u32;

    fn flags(&self) -> LightFlags;

    fn power(&self) -> Spectrum;

    fn le(&self, _ray: &Ray) -> Spectrum {
        Spectrum::black()
    }
}

pub trait AreaLight: Light {
    fn l(&self, si: &Interaction, w: &Vector3f) -> Spectrum;
}
