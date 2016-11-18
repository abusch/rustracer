use std::f32;
use std::f32::consts::*;
use na::Norm;

use Point;
use Vector;
use colour::Colourf;
use interaction::SurfaceInteraction;
use ray::Ray;
use scene::Scene;

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
                 sample: (f32, f32))
                 -> (Colourf, Vector, f32, VisibilityTester);

    fn preprocess(&mut self, scene: &Scene) {}

    fn n_samples(&self) -> u32;

    fn flags(&self) -> LightFlags;

    fn power(&self) -> Colourf;
}

#[derive(Debug)]
pub struct PointLight {
    pub pos: Point,
    pub emission_colour: Colourf,
}

impl PointLight {
    pub fn new(p: Point, ec: Colourf) -> PointLight {
        PointLight {
            pos: p,
            emission_colour: ec,
        }
    }
}

impl Light for PointLight {
    fn sample_li(&self,
                 isect: &SurfaceInteraction,
                 wo: &Vector,
                 sample: (f32, f32))
                 -> (Colourf, Vector, f32, VisibilityTester) {
        let wi = self.pos - isect.p;
        let r2 = wi.norm_squared();
        let l_i = self.emission_colour / (4.0 * PI * r2);
        let vt = VisibilityTester::new(isect.spawn_ray_to(&self.pos));

        (l_i, wi.normalize(), 1.0, vt)
    }

    fn n_samples(&self) -> u32 {
        1
    }

    fn flags(&self) -> LightFlags {
        DELTA_POSITION
    }

    fn power(&self) -> Colourf {
        4.0 * PI * self.emission_colour
    }
}

#[derive(Debug)]
pub struct DistantLight {
    pub dir: Vector,
    pub emission_colour: Colourf,
    w_center: Point,
    w_radius: f32,
}

impl DistantLight {
    pub fn new(dir: Vector, ec: Colourf) -> DistantLight {
        DistantLight {
            dir: dir.normalize(),
            emission_colour: ec,
            w_center: Point::new(0.0, 0.0, 0.0),
            w_radius: 0.0,
        }
    }
}

impl Light for DistantLight {
    fn preprocess(&mut self, scene: &Scene) {
        let (w_center, w_radius) = scene.world_bounds().bounding_sphere();
        self.w_center = w_center;
        self.w_radius = w_radius;
    }

    fn sample_li(&self,
                 isect: &SurfaceInteraction,
                 _wo: &Vector,
                 _sample: (f32, f32))
                 -> (Colourf, Vector, f32, VisibilityTester) {
        let p_outside = isect.p - self.dir * (2.0 * self.w_radius);
        (self.emission_colour,
         -self.dir,
         1.0,
         VisibilityTester::new(isect.spawn_ray_to(&p_outside)))
    }

    fn n_samples(&self) -> u32 {
        1
    }

    fn flags(&self) -> LightFlags {
        DELTA_DIRECTION
    }

    fn power(&self) -> Colourf {
        self.emission_colour * PI * self.w_radius * self.w_radius
    }
}
