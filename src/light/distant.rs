use std::f32::consts::PI;
use std::sync::Arc;

use num::Zero;
use uuid::Uuid;

use {Point2f, Point3f, Transform, Vector3f};
use interaction::{Interaction, SurfaceInteraction};
use light::{Light, LightFlags, VisibilityTester};
use paramset::ParamSet;
use scene::Scene;
use spectrum::Spectrum;

#[derive(Debug)]
pub struct DistantLight {
    pub id: Uuid,
    pub dir: Vector3f,
    pub emission_colour: Spectrum,
    w_center: Point3f,
    w_radius: f32,
}

impl DistantLight {
    pub fn new(dir: Vector3f, ec: Spectrum) -> DistantLight {
        DistantLight {
            id: Uuid::new_v4(),
            dir: dir.normalize(),
            emission_colour: ec,
            w_center: Point3f::new(0.0, 0.0, 0.0),
            w_radius: 100.0, // TODO
        }
    }

    pub fn create(l2w: &Transform, params: &mut ParamSet) -> Arc<Light + Send + Sync> {
        let L = params.find_one_spectrum("L", Spectrum::white());
        let scale = params.find_one_spectrum("scale", Spectrum::white());
        let from = params.find_one_point3f("from", Point3f::zero());
        let to = params.find_one_point3f("to", Point3f::new(0.0, 0.0, 1.0));
        let dir = from - to;
        Arc::new(DistantLight::new(l2w * &dir, L * scale))
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

    fn sample_li(
        &self,
        isect: &SurfaceInteraction,
        _u: &Point2f,
    ) -> (Spectrum, Vector3f, f32, VisibilityTester) {
        let p_outside = isect.p + self.dir * (2.0 * self.w_radius);
        (
            self.emission_colour,
            self.dir,
            1.0,
            VisibilityTester::new(isect.into(), Interaction::from_point(&p_outside)),
        )
    }

    fn pdf_li(&self, _si: &SurfaceInteraction, _wi: &Vector3f) -> f32 {
        0.0
    }

    fn n_samples(&self) -> u32 {
        1
    }

    fn flags(&self) -> LightFlags {
        LightFlags::DELTA_DIRECTION
    }

    fn power(&self) -> Spectrum {
        self.emission_colour * PI * self.w_radius * self.w_radius
    }
}
