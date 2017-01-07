use std::f32::consts::PI;

use na::Norm;
use uuid::Uuid;

use {Point, Vector3f, Point2f};
use interaction::{Interaction, SurfaceInteraction};
use light::{Light, LightFlags, VisibilityTester, DELTA_DIRECTION};
use scene::Scene;
use spectrum::Spectrum;

#[derive(Debug)]
pub struct DistantLight {
    pub id: Uuid,
    pub dir: Vector3f,
    pub emission_colour: Spectrum,
    w_center: Point,
    w_radius: f32,
}

impl DistantLight {
    pub fn new(dir: Vector3f, ec: Spectrum) -> DistantLight {
        DistantLight {
            id: Uuid::new_v4(),
            dir: dir.normalize(),
            emission_colour: ec,
            w_center: Point::new(0.0, 0.0, 0.0),
            w_radius: 100.0, // TODO
        }
    }
}

impl Light for DistantLight {
    fn id(&self) -> Uuid {
        self.id
    }

    fn preprocess(&mut self, scene: &Scene) {
        let (w_center, w_radius) = scene.world_bounds().bounding_sphere();
        self.w_center = w_center;
        self.w_radius = w_radius;
    }

    fn sample_li(&self,
                 isect: &SurfaceInteraction,
                 _u: &Point2f)
                 -> (Spectrum, Vector3f, f32, VisibilityTester) {
        let p_outside = isect.p - self.dir * (2.0 * self.w_radius);
        (self.emission_colour,
         -self.dir,
         1.0,
         VisibilityTester::new(isect.into(), Interaction::from_point(&p_outside)))
    }

    fn pdf_li(&self, _si: &SurfaceInteraction, _wi: &Vector3f) -> f32 {
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
