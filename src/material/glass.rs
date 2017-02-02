use std::sync::Arc;

use bsdf::{BSDF, BxDF, FresnelSpecular, SpecularTransmission, SpecularReflection, Fresnel};
use interaction::SurfaceInteraction;
use material::{Material, TransportMode};
use spectrum::Spectrum;
use texture::{Texture, ConstantTexture};

pub struct GlassMaterial {
    kr: Arc<Texture<Spectrum> + Send + Sync>,
    kt: Arc<Texture<Spectrum> + Send + Sync>,
    u_roughness: Arc<Texture<f32> + Send + Sync>,
    v_roughness: Arc<Texture<f32> + Send + Sync>,
    index: Arc<Texture<f32> + Send + Sync>,
    remap_roughness: bool,
}

impl GlassMaterial {
    pub fn new() -> GlassMaterial {
        GlassMaterial {
            kr: Arc::new(ConstantTexture::new(Spectrum::white())),
            kt: Arc::new(ConstantTexture::new(Spectrum::white())),
            u_roughness: Arc::new(ConstantTexture::new(0.0)),
            v_roughness: Arc::new(ConstantTexture::new(0.0)),
            index: Arc::new(ConstantTexture::new(1.5)),
            remap_roughness: true,
        }
    }
}


impl Material for GlassMaterial {
    fn compute_scattering_functions(&self,
                                    isect: &mut SurfaceInteraction,
                                    mode: TransportMode,
                                    allow_multiple_lobes: bool) {
        let eta = self.index.evaluate(isect);
        let u_rough = self.u_roughness.evaluate(isect);
        let v_rough = self.v_roughness.evaluate(isect);
        let r = self.kr.evaluate(isect);
        let t = self.kt.evaluate(isect);

        let mut bxdfs: Vec<Box<BxDF + Send + Sync>> = Vec::new();

        if !r.is_black() || !t.is_black() {
            let is_specular = u_rough == 0.0 && v_rough == 0.0;
            if is_specular && allow_multiple_lobes {
                bxdfs.push(Box::new(FresnelSpecular::new()));
            } else {
                // TODO remap roughness
                if !r.is_black() {
                    let fresnel = Box::new(Fresnel::dielectric(1.0, eta));
                    if is_specular {
                        let bxdf = Box::new(SpecularReflection::new(r, fresnel));
                        bxdfs.push(bxdf);
                    } else {
                        // TODO
                        unimplemented!();
                    }
                }
                if !t.is_black() {
                    if is_specular {
                        let bxdf = Box::new(SpecularTransmission::new(t, 1.0, eta));
                        bxdfs.push(bxdf);
                    } else {
                        // TODO
                        unimplemented!();
                    }
                }
            }
        }

        isect.bsdf = Some(Arc::new(BSDF::new(isect, eta, bxdfs)));
    }
}
