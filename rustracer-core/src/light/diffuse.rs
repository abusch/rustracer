use std::f32::consts::PI;
use std::sync::Arc;

use crate::interaction::Interaction;
use crate::light::{AreaLight, Light, LightFlags, VisibilityTester};
use crate::paramset::ParamSet;
use crate::shapes::Shape;
use crate::spectrum::Spectrum;
use crate::{Point2f, Transform, Vector3f};

#[derive(Debug)]
pub struct DiffuseAreaLight {
    id: u32,
    l_emit: Spectrum,
    shape: Arc<dyn Shape>,
    n_samples: u32,
    two_sided: bool,
    area: f32,
}

impl DiffuseAreaLight {
    pub fn new(
        l_emit: Spectrum,
        shape: Arc<dyn Shape>,
        n_samples: u32,
        two_sided: bool,
    ) -> DiffuseAreaLight {
        let area = shape.area();
        DiffuseAreaLight {
            id: super::get_next_id(),
            l_emit,
            shape,
            n_samples,
            two_sided,
            area,
        }
    }

    pub fn create(
        _light2world: &Transform,
        ps: &ParamSet,
        shape: Arc<dyn Shape>,
    ) -> Arc<DiffuseAreaLight> {
        let L = ps.find_one_spectrum("L", Spectrum::white());
        let sc = ps.find_one_spectrum("scale", Spectrum::white());
        let nsamples = ps.find_one_int("nsamples", 1);
        let nsamples = ps.find_one_int("samples", nsamples);
        let two_sided = ps.find_one_bool("twosided", false);

        Arc::new(Self::new(L * sc, shape, nsamples as u32, two_sided))
    }
}

impl Light for DiffuseAreaLight {
    fn id(&self) -> u32 {
        self.id
    }

    fn sample_li(
        &self,
        si: &Interaction,
        u: &Point2f,
    ) -> (Spectrum, Vector3f, f32, VisibilityTester) {
        let (p_shape, pdf) = self.shape.sample_si(si, u);
        assert!(!p_shape.p.x.is_nan() && !p_shape.p.y.is_nan() && !p_shape.p.z.is_nan());
        let wi = (p_shape.p - si.p).normalize();
        let vis = VisibilityTester::new(*si, p_shape);

        (self.l(&p_shape, &(-wi)), wi, pdf, vis)
    }

    fn pdf_li(&self, si: &Interaction, wi: &Vector3f) -> f32 {
        self.shape.pdf_wi(si, wi)
    }

    fn n_samples(&self) -> u32 {
        self.n_samples
    }

    fn flags(&self) -> LightFlags {
        LightFlags::AREA
    }

    fn power(&self) -> Spectrum {
        let factor = if self.two_sided { 2.0 } else { 1.0 };
        factor * self.l_emit * PI * self.area
    }
}

impl AreaLight for DiffuseAreaLight {
    fn l(&self, si: &Interaction, w: &Vector3f) -> Spectrum {
        if self.two_sided || si.n.dot(w) > 0.0 {
            self.l_emit
        } else {
            Spectrum::black()
        }
    }
}
