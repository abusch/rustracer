use std::sync::Arc;
use std::fmt::Debug;

use Transform;
use interaction::SurfaceInteraction;
use paramset::TextureParams;
use spectrum::Spectrum;
use texture::{ConstantTexture, Texture, TextureMapping2D, UVMapping2D};

#[derive(Debug)]
pub struct CheckerboardTexture<T> {
    tex1: Arc<Texture<T> + Send + Sync>,
    tex2: Arc<Texture<T> + Send + Sync>,
    mapping: Box<TextureMapping2D + Send + Sync>,
}

impl<T> CheckerboardTexture<T> {
    pub fn new(tex1: Arc<Texture<T> + Send + Sync>,
               tex2: Arc<Texture<T> + Send + Sync>,
               mapping: Box<TextureMapping2D + Send + Sync>)
               -> CheckerboardTexture<T> {
        CheckerboardTexture {
            tex1: tex1,
            tex2: tex2,
            mapping: mapping,
        }
    }
}

impl CheckerboardTexture<Spectrum> {
    pub fn bw() -> CheckerboardTexture<Spectrum> {
        CheckerboardTexture::new(Arc::new(ConstantTexture::new(Spectrum::black())),
                                 Arc::new(ConstantTexture::new(Spectrum::white())),
                                 Box::new(UVMapping2D::new(10.0, 10.0, 0.0, 0.0)))
    }

    pub fn create_spectrum(_tex2world: &Transform,
                           tp: &mut TextureParams)
                           -> CheckerboardTexture<Spectrum> {
        let dim = tp.find_int("dimension", 2);
        if dim != 2 && dim != 3 {
            panic!("{} dimensional checkerboard texture not supported", dim);
        }
        let tex1 = tp.get_spectrum_texture("tex1", &Spectrum::white());
        let tex2 = tp.get_spectrum_texture("tex2", &Spectrum::black());
        if dim == 2 {
            // Initialize 2D texture mapping `map` from `tp`
            let typ = tp.find_string("mapping", "uv");
            let map = if typ == "uv" {
                let su = tp.find_float("uscale", 1.0);
                let sv = tp.find_float("vscale", 1.0);
                let du = tp.find_float("udelta", 0.0);
                let dv = tp.find_float("vdelta", 0.0);
                UVMapping2D::new(su, sv, du, dv)
            } else if typ == "spherical" {
                unimplemented!()
            } else if typ == "cylindrical" {
                unimplemented!()
            } else if typ == "planar" {
                unimplemented!()
            } else {
                error!("2D texture mapping {} unknown", typ);
                UVMapping2D::new(1.0, 1.0, 0.0, 0.0)
            };

            // Compute `aaMethod` for `CheckerboardTexture`
            let _aa = tp.find_string("aamode", "closedform");
            // TODO finish aamode
            CheckerboardTexture::new(tex1, tex2, Box::new(map))
        } else {
            unimplemented!()
        }
    }
}

impl<T: Debug> Texture<T> for CheckerboardTexture<T> {
    fn evaluate(&self, si: &SurfaceInteraction) -> T {
        let (st, _dstdx, _dstdy) = self.mapping.map(si);
        if (st.x.floor() as u32 + st.y.floor() as u32) % 2 == 0 {
            self.tex1.evaluate(si)
        } else {
            self.tex2.evaluate(si)
        }
        // TODO implement closeform antialiasing
    }
}
