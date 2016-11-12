use bsdf::{BSDF, BxDF, FresnelConductor, SpecularReflection};
use colour::Colourf;
use interaction::SurfaceInteraction;

pub struct Material {
    pub surface_colour: Colourf,
    pub transparency: f32,
    pub reflection: f32,
    pub bxdfs: Vec<Box<BxDF + Send + Sync>>,
}

impl Material {
    pub fn new(sc: Colourf, rf: f32, tr: f32) -> Material {
        Material {
            surface_colour: sc,
            transparency: tr,
            reflection: rf,
            bxdfs: vec![Box::new(SpecularReflection::new(Colourf::rgb(1.0, 0.0, 0.0),
                                                         Box::new(FresnelConductor::new(
                                                                 Colourf::white(),
                                                                 Colourf::rgb(0.155265, 0.116723, 0.138381),
                                                                 Colourf::rgb(4.82835, 3.12225, 2.14696),
                                                                 ))))],
        }
    }

    pub fn bsdf(&self, isect: &SurfaceInteraction) -> BSDF {
        BSDF::new(isect, 1.5, &self.bxdfs[..])
    }
}
