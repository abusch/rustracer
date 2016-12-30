use std::f32::consts::PI;

use na::Norm;
use uuid::Uuid;

use {Point, Vector, Point2f};
use light::{Light, LightFlags, VisibilityTester, DELTA_POSITION};
use spectrum::Spectrum;
use interaction::{Interaction, SurfaceInteraction};

#[derive(Debug)]
pub struct PointLight {
    pub id: Uuid,
    pub pos: Point,
    pub emission_colour: Spectrum,
}

impl PointLight {
    pub fn new(p: Point, ec: Spectrum) -> PointLight {
        PointLight {
            id: Uuid::new_v4(),
            pos: p,
            emission_colour: ec,
        }
    }
}

impl Light for PointLight {
    fn id(&self) -> Uuid {
        self.id
    }

    fn sample_li(&self,
                 isect: &SurfaceInteraction,
                 _u: &Point2f)
                 -> (Spectrum, Vector, f32, VisibilityTester) {
        let wi = self.pos - isect.p;
        let r2 = wi.norm_squared();
        let l_i = self.emission_colour / (4.0 * PI * r2);
        let vt = VisibilityTester::new(isect.into(), Interaction::from_point(&self.pos));

        (l_i, wi.normalize(), 1.0, vt)
    }

    fn pdf_li(&self, _si: &SurfaceInteraction, _wi: &Vector) -> f32 {
        0.0
    }

    fn n_samples(&self) -> u32 {
        1
    }

    fn flags(&self) -> LightFlags {
        DELTA_POSITION
    }

    fn power(&self) -> Spectrum {
        4.0 * PI * self.emission_colour
    }
}
