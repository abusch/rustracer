use std::f32::consts::PI;
use std::sync::Arc;

use na::{Dot, Norm};

use {Vector, Point2f};
use light::{AreaLight, Light, LightFlags, VisibilityTester, AREA};
use shapes::Shape;
use spectrum::Spectrum;
use interaction::{SurfaceInteraction, Interaction};


pub struct DiffuseAreaLight {
    l_emit: Spectrum,
    shape: Arc<Shape + Send + Sync>,
    n_samples: u32,
    area: f32,
}

impl DiffuseAreaLight {
    pub fn new(l_emit: Spectrum,
               shape: Arc<Shape + Send + Sync>,
               n_samples: u32)
               -> DiffuseAreaLight {
        let area = shape.area();
        DiffuseAreaLight {
            l_emit: l_emit,
            shape: shape,
            n_samples: n_samples,
            area: area,
        }
    }
}

impl Light for DiffuseAreaLight {
    fn sample_li(&self,
                 si: &SurfaceInteraction,
                 _wo: &Vector,
                 u: &Point2f)
                 -> (Spectrum, Vector, f32, VisibilityTester) {
        let (p_shape, pdf) = self.shape.sample_si(&si.into(), u);
        assert!(!p_shape.p.x.is_nan() && !p_shape.p.y.is_nan() && !p_shape.p.z.is_nan());
        let wi = (p_shape.p - si.p).normalize();
        let vis = VisibilityTester::new(si.into(), p_shape);

        (self.l(&p_shape, &(-wi)), wi, pdf, vis)
    }

    fn pdf_li(&self, si: &SurfaceInteraction, wi: &Vector) -> f32 {
        self.shape.pdf_wi(si, wi)
    }

    fn n_samples(&self) -> u32 {
        self.n_samples
    }

    fn flags(&self) -> LightFlags {
        AREA
    }

    fn power(&self) -> Spectrum {
        self.l_emit * PI * self.area
    }
}

impl AreaLight for DiffuseAreaLight {
    fn l(&self, si: &Interaction, w: &Vector) -> Spectrum {
        if si.n.dot(w) > 0.0 {
            self.l_emit
        } else {
            Spectrum::black()
        }
    }
}
