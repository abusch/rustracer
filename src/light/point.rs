use std::sync::Arc;
use std::f32::consts::PI;

use uuid::Uuid;

use {Point3f, Vector3f, Point2f, Transform};
use interaction::{Interaction, SurfaceInteraction};
use light::{Light, LightFlags, VisibilityTester};
use paramset::ParamSet;
use spectrum::Spectrum;

#[derive(Debug)]
pub struct PointLight {
    pub id: Uuid,
    pub pos: Point3f,
    pub emission_colour: Spectrum,
}

impl PointLight {
    pub fn new(p: Point3f, ec: Spectrum) -> PointLight {
        PointLight {
            id: Uuid::new_v4(),
            pos: p,
            emission_colour: ec,
        }
    }

    pub fn create(l2w: &Transform, params: &mut ParamSet) -> Arc<Light + Send + Sync> {
        let I = params.find_one_spectrum("I", Spectrum::white());
        let scale = params.find_one_spectrum("scale", Spectrum::white());
        let p = params.find_one_point3f("from", Point3f::origin());

        let t = &Transform::translate(Vector3f::new(p.x, p.y, p.z)) * l2w;
        Arc::new(PointLight::new(&t * &Point3f::origin(), I * scale))
    }
}

impl Light for PointLight {
    fn id(&self) -> Uuid {
        self.id
    }

    fn sample_li(&self,
                 isect: &SurfaceInteraction,
                 _u: &Point2f)
                 -> (Spectrum, Vector3f, f32, VisibilityTester) {
        let wi = self.pos - isect.p;
        let r2 = wi.norm_squared();
        let l_i = self.emission_colour / (4.0 * PI * r2);
        let vt = VisibilityTester::new(isect.into(), Interaction::from_point(&self.pos));

        (l_i, wi.normalize(), 1.0, vt)
    }

    fn pdf_li(&self, _si: &SurfaceInteraction, _wi: &Vector3f) -> f32 {
        0.0
    }

    fn n_samples(&self) -> u32 {
        1
    }

    fn flags(&self) -> LightFlags {
        LightFlags::DELTA_POSITION
    }

    fn power(&self) -> Spectrum {
        4.0 * PI * self.emission_colour
    }
}
