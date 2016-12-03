use std::f32::consts::PI;

use na::Norm;

use {Point, Vector, Point2f};
use interaction::SurfaceInteraction;
use light::{Light, LightFlags, VisibilityTester, DELTA_DIRECTION};
use scene::Scene;
use spectrum::Spectrum;

#[derive(Debug)]
pub struct DistantLight {
    pub dir: Vector,
    pub emission_colour: Spectrum,
    w_center: Point,
    w_radius: f32,
}

impl DistantLight {
    pub fn new(dir: Vector, ec: Spectrum) -> DistantLight {
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
                 _u: &Point2f)
                 -> (Spectrum, Vector, f32, VisibilityTester) {
        let p_outside = isect.p - self.dir * (2.0 * self.w_radius);
        (self.emission_colour,
         -self.dir,
         1.0,
         // TODO can't use self.w_radius as I've disabled preprocess for now...
         // VisibilityTester::new(isect.spawn_ray_to(&p_outside)))
         VisibilityTester::new(isect.spawn_ray(&(-self.dir))))
    }

    fn pdf_li(&self, _si: &SurfaceInteraction, _wi: &Vector) -> f32 {
        0.0
    }

    fn n_samples(&self) -> u32 {
        1
    }

    fn flags(&self) -> LightFlags {
        DELTA_DIRECTION
    }

    fn power(&self) -> Spectrum {
        self.emission_colour * PI * self.w_radius * self.w_radius
    }
}
