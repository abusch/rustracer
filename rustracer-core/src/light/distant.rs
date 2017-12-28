use std::f32::consts::PI;
use std::sync::Arc;

use num::Zero;
use parking_lot::RwLock;

use {Point2f, Point3f, Transform, Vector3f};
use interaction::Interaction;
use light::{Light, LightFlags, VisibilityTester};
use paramset::ParamSet;
use scene::Scene;
use spectrum::Spectrum;

#[derive(Debug)]
pub struct DistantLight {
    id: u32,
    dir: Vector3f,
    emission_colour: Spectrum,
    w_center: RwLock<Point3f>,
    w_radius: RwLock<f32>,
}

impl DistantLight {
    pub fn new(dir: Vector3f, ec: Spectrum) -> DistantLight {
        DistantLight {
            id: super::get_next_id(),
            dir: dir.normalize(),
            emission_colour: ec,
            w_center: RwLock::new(Point3f::new(0.0, 0.0, 0.0)),
            w_radius: RwLock::new(0.0),
        }
    }

    pub fn create(l2w: &Transform, params: &mut ParamSet) -> Arc<Light> {
        let L = params.find_one_spectrum("L", Spectrum::white());
        let scale = params.find_one_spectrum("scale", Spectrum::white());
        let from = params.find_one_point3f("from", Point3f::zero());
        let to = params.find_one_point3f("to", Point3f::new(0.0, 0.0, 1.0));
        let dir = from - to;
        Arc::new(DistantLight::new(l2w * &dir, L * scale))
    }
}

impl Light for DistantLight {
    fn id(&self) -> u32 {
        self.id
    }

    fn preprocess(&self, scene: &Scene) {
        let (w_center, w_radius) = scene.world_bounds().bounding_sphere();
        let mut wc = self.w_center.write();
        *wc = w_center;
        let mut wr = self.w_radius.write();
        *wr = w_radius;
    }

    fn sample_li(&self,
                 isect: &Interaction,
                 _u: &Point2f)
                 -> (Spectrum, Vector3f, f32, VisibilityTester) {
        let wr = self.w_radius.read();
        let p_outside = isect.p + self.dir * (2.0 * *wr);
        (self.emission_colour,
         self.dir,
         1.0,
         VisibilityTester::new(*isect, Interaction::from_point(&p_outside)))
    }

    fn pdf_li(&self, _si: &Interaction, _wi: &Vector3f) -> f32 {
        0.0
    }

    fn n_samples(&self) -> u32 {
        1
    }

    fn flags(&self) -> LightFlags {
        LightFlags::DELTA_DIRECTION
    }

    fn power(&self) -> Spectrum {
        let wr = self.w_radius.read();
        self.emission_colour * PI * *wr * *wr
    }
}
