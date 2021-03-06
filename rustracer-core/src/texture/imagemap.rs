use num::Zero;
use std::fmt::Debug;
use std::ops::{AddAssign, Div, Mul};
use std::path::Path;
use std::sync::Arc;

use log::{debug, info, warn};

use crate::bounds::Bounds2i;
use crate::fileutil;
use crate::imageio::read_image;
use crate::interaction::SurfaceInteraction;
use crate::mipmap::{MIPMap, WrapMode};
use crate::paramset::TextureParams;
use crate::spectrum::Spectrum;
use crate::texture::{Texture, TextureMapping2D, UVMapping2D};
use crate::transform::Transform;
use crate::{Clampable, Point2i};

#[derive(Debug)]
pub struct ImageTexture<T> {
    mapping: Box<dyn TextureMapping2D>,
    mipmap: Arc<MIPMap<T>>,
}

impl<T> ImageTexture<T>
where
    T: Zero,
    T: Clone,
    T: Copy,
    T: Clampable,
    T: Debug,
    T: AddAssign<T>,
    T: Mul<f32, Output = T>,
    T: Div<f32, Output = T>,
    T: Sized,
    T: Send + Sync,
{
    pub fn new<F: Fn(&Spectrum) -> T>(
        path: &Path,
        wrap_mode: WrapMode,
        trilerp: bool,
        max_aniso: f32,
        scale: f32,
        gamma: bool,
        map: Box<dyn TextureMapping2D>,
        convert: F,
    ) -> ImageTexture<T> {
        debug!("Loading texture {}", path.display());
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

        let converted_texels: Vec<T> = texels
            .iter()
            .map(|p| {
                let s = if gamma {
                    scale * p.inverse_gamma_correct()
                } else {
                    scale * *p
                };
                convert(&s)
            })
            .collect();

        let mipmap = Arc::new(MIPMap::new(
            res,
            &converted_texels[..],
            trilerp,
            max_aniso,
            wrap_mode,
        ));
        ImageTexture {
            mapping: map,
            mipmap,
        }
    }
}

impl ImageTexture<Spectrum> {
    pub fn create(_tex2world: &Transform, tp: &TextureParams<'_>) -> ImageTexture<Spectrum> {
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
            fileutil::has_extension(&filename, "tga") || fileutil::has_extension(&filename, "png"),
        );

        Self::new(
            Path::new(&filename),
            wrap_mode,
            trilerp,
            max_aniso,
            scale,
            gamma,
            Box::new(map),
            convert_to_spectrum,
        )
    }

    pub fn dump_mipmap(&self) {
        info!("Dumping MIPMap levels for debugging...");
        self.mipmap
            .pyramid
            .iter()
            .enumerate()
            .for_each(|(i, level)| {
                let mut buf = Vec::new();
                for y in 0..level.v_size() {
                    for x in 0..level.u_size() {
                        let p = level[(x, y)];
                        buf.push(p[0]);
                        buf.push(p[1]);
                        buf.push(p[2]);
                    }
                }
                crate::imageio::write_image(
                    format!("mipmap_level_{}.png", i),
                    &buf[..],
                    &Bounds2i::from_elements(0, 0, level.u_size() as i32, level.v_size() as i32),
                    Point2i::new(level.u_size() as i32, level.v_size() as i32),
                )
                .unwrap();
            });
    }
}

impl ImageTexture<f32> {
    pub fn create(_tex2world: &Transform, tp: &TextureParams<'_>) -> ImageTexture<f32> {
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
            fileutil::has_extension(&filename, "tga") || fileutil::has_extension(&filename, "png"),
        );

        Self::new(
            Path::new(&filename),
            wrap_mode,
            trilerp,
            max_aniso,
            scale,
            gamma,
            Box::new(map),
            convert_to_float,
        )
    }
}
fn convert_to_spectrum(from: &Spectrum) -> Spectrum {
    *from
}

fn convert_to_float(from: &Spectrum) -> f32 {
    from.y()
}

impl<T> Texture<T> for ImageTexture<T>
where
    T: Zero,
    T: Clone,
    T: Copy,
    T: Send,
    T: Sync,
    T: Clampable,
    T: Debug,
    T: AddAssign<T>,
    T: Mul<f32, Output = T>,
    T: Div<f32, Output = T>,
    T: Sized,
{
    fn evaluate(&self, si: &SurfaceInteraction<'_, '_>) -> T {
        let (st, dstdx, dstdy) = self.mapping.map(si);
        self.mipmap.lookup_diff(st, dstdx, dstdy)
    }
}
