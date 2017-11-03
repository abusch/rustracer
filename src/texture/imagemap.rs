use std::sync::Arc;
use std::path::Path;

use Point2i;
use fileutil;
use interaction::SurfaceInteraction;
use imageio::read_image;
use mipmap::{MIPMap, WrapMode};
use spectrum::Spectrum;
use texture::{Texture, TextureMapping2D, UVMapping2D};
use paramset::TextureParams;
use transform::Transform;

#[derive(Debug)]
pub struct ImageTexture {
    mapping: Box<TextureMapping2D + Send + Sync>,
    mipmap: Arc<MIPMap<Spectrum>>,
}

impl ImageTexture {
    pub fn new(
        path: &Path,
        wrap_mode: WrapMode,
        trilerp: bool,
        max_aniso: f32,
        scale: f32,
        gamma: bool,
        map: Box<TextureMapping2D + Send + Sync>,
    ) -> ImageTexture {
        info!("Loading texture {}", path.display());
        let (res, texels) = match read_image(path) {
            Ok((mut pixels, res)) => {
                // Flip image in y; texture coordinate space has (0,0) at the lower
                // left corner.
                for y in 0..res.y / 2 {
                    for x in 0..res.x {
                        let o1 = (y * res.x + x) as usize;
                        let o2 = ((res.y - 1 - y) * res.x + x) as usize;
                        pixels.swap(o1, o2);
                    }
                }

                (res, pixels)
            }
            Err(e) => {
                warn!(
                    "Could not open texture file. Using grey texture instead: {}",
                    e
                );
                (Point2i::new(1, 1), vec![Spectrum::grey(0.18)])
            }
        };

        let converted_texels: Vec<Spectrum> = texels.iter().map(|p|{
            if gamma {
                scale * p.inverse_gamma_correct()
            } else {
                scale * *p
            }
        }).collect();

        ImageTexture {
            mapping: map,
            mipmap: Arc::new(MIPMap::new(
                &res,
                &converted_texels[..],
                trilerp,
                max_aniso,
                wrap_mode,
            )),
        }
    }

    pub fn create(_tex2world: &Transform, tp: &mut TextureParams) -> ImageTexture {
        let typ = tp.find_string("mapping", "uv");
        let map = if typ == "uv" {
            let su = tp.find_float("uscale", 1.0);
            let sv = tp.find_float("vscale", 1.0);
            let du = tp.find_float("udelta", 0.0);
            let dv = tp.find_float("vdelta", 0.0);

            UVMapping2D::new(su, sv, du, dv)
        } else {
            unimplemented!()
        };
        let max_aniso = tp.find_float("maxanisotropy", 8.0);
        let trilerp = tp.find_bool("trilinear", false);
        let wrap = tp.find_string("wrap", "repeat");
        let wrap_mode = if wrap == "black" {
            WrapMode::Black
        } else if wrap == "clamp" {
            WrapMode::Clamp
        } else {
            WrapMode::Repeat
        };
        let scale = tp.find_float("scale", 1.0);
        let filename = tp.find_filename("filename", "");
        let gamma = tp.find_bool(
            "gamma",
            fileutil::has_extension(&filename, "tga")
                || fileutil::has_extension(&filename, "png"),
        );

        Self::new(
            Path::new(&filename),
            wrap_mode,
            trilerp,
            max_aniso,
            scale,
            gamma,
            Box::new(map),
        )
    }
}

impl Texture<Spectrum> for ImageTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> Spectrum {
        let st = self.mapping.map(si);
        // TODO Call correct lookup method once we have ray differentials
        self.mipmap.lookup(&st, 0.0)
    }
}
