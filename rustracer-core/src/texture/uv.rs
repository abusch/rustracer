use interaction::SurfaceInteraction;
use paramset::TextureParams;
use spectrum::Spectrum;
use texture::{Texture, TextureMapping2D, UVMapping2D};
use Transform;

#[derive(Debug)]
pub struct UVTexture {
    mapping: Box<dyn TextureMapping2D>,
}

impl UVTexture {
    pub fn new() -> UVTexture {
        UVTexture {
            mapping: Box::new(UVMapping2D::new(1.0, 1.0, 0.0, 0.0)),
        }
    }

    pub fn create_spectrum(_tex2world: &Transform, tp: &TextureParams) -> UVTexture {
        let typ = tp.find_string("mapping", "uv");
        let mapping = if typ == "uv" {
            let su = tp.find_float("uscale", 1.0);
            let sv = tp.find_float("vscale", 1.0);
            let du = tp.find_float("udelta", 0.0);
            let dv = tp.find_float("vdelta", 0.0);
            Box::new(UVMapping2D::new(su, sv, du, dv))
        } else if typ == "spherical" {
            unimplemented!()
        } else if typ == "cylindrical" {
            unimplemented!()
        } else if typ == "planar" {
            unimplemented!()
        } else {
            error!("2D texture mapping \"{}\" unknown.", typ);
            Box::new(UVMapping2D::new(1.0, 1.0, 0.0, 0.0))
        };

        UVTexture { mapping }
    }
}

impl Default for UVTexture {
    fn default() -> Self {
        Self::new()
    }
}

impl Texture<Spectrum> for UVTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> Spectrum {
        let (st, _dstdx, _dstdy) = self.mapping.map(si);
        Spectrum::rgb(st[0] - st[0].floor(), st[1] - st[1].floor(), 0.0)
    }
}
