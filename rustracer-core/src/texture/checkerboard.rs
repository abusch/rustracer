use std::sync::Arc;
use std::fmt::Debug;
use std::ops::{Add, Mul};

use Transform;
use interaction::SurfaceInteraction;
use paramset::TextureParams;
use spectrum::Spectrum;
use texture::{Texture, TextureMapping2D, UVMapping2D};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum AAMethod {
    None,
    ClosedForm,
}

#[derive(Debug)]
pub struct CheckerboardTexture<T> {
    tex1: Arc<Texture<T> + Send + Sync>,
    tex2: Arc<Texture<T> + Send + Sync>,
    mapping: Box<TextureMapping2D + Send + Sync>,
    aa_method: AAMethod,
}

impl<T> CheckerboardTexture<T> {
    pub fn new(tex1: Arc<Texture<T> + Send + Sync>,
               tex2: Arc<Texture<T> + Send + Sync>,
               mapping: Box<TextureMapping2D + Send + Sync>,
               aa_method: AAMethod)
               -> CheckerboardTexture<T> {
        CheckerboardTexture {
            tex1,
            tex2,
            mapping,
            aa_method,
        }
    }
}

impl CheckerboardTexture<Spectrum> {
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
            let aa = tp.find_string("aamode", "closedform");
            let aa_method = if aa == "none" {
                AAMethod::None
            } else if aa == "closedform" {
                AAMethod::ClosedForm
            } else {
                warn!("Unknown aamethod \"{}\" found for CheckerboardTexture. Using closedform instead",
                      aa);
                AAMethod::ClosedForm
            };
            CheckerboardTexture::new(tex1, tex2, Box::new(map), aa_method)
        } else {
            unimplemented!()
        }
    }
}

impl<T> Texture<T> for CheckerboardTexture<T>
    where T: Debug,
          T: Mul<f32, Output = T>,
          T: Add<Output = T>
{
    fn evaluate(&self, si: &SurfaceInteraction) -> T {
        let (st, dstdx, dstdy) = self.mapping.map(si);
        match self.aa_method {
            AAMethod::None => {
                if (st.x.floor() as u32 + st.y.floor() as u32) % 2 == 0 {
                    self.tex1.evaluate(si)
                } else {
                    self.tex2.evaluate(si)
                }
            }
            AAMethod::ClosedForm => {
                // Compute closed-form box-filtered _Checkerboard2DTexture_ value

                // Evaluate single check if filter is entirely inside one of them
                let ds = f32::max(f32::abs(dstdx[0]), f32::abs(dstdy[0]));
                let dt = f32::max(f32::abs(dstdx[1]), f32::abs(dstdy[1]));
                let s0 = st[0] - ds;
                let s1 = st[0] + ds;
                let t0 = st[1] - dt;
                let t1 = st[1] + dt;
                if f32::floor(s0) == f32::floor(s1) && f32::floor(t0) == f32::floor(t1) {
                    // Point sample _Checkerboard2DTexture_
                    if (f32::floor(st[0]) as i32 + f32::floor(st[1]) as i32) % 2 == 0 {
                        return self.tex1.evaluate(si);
                    } else {
                        return self.tex2.evaluate(si);
                    }
                }

                // Apply box filter to checkerboard region
                fn bump_int(x: f32) -> i32 {
                    return (f32::floor(x / 2.0) +
                            2.0 * f32::max(x / 2.0 - f32::floor(x / 2.0) - 0.5, 0.0)) as
                           i32;
                }
                let sint = (bump_int(s1) - bump_int(s0)) as f32 / (2.0 * ds);
                let tint = (bump_int(t1) - bump_int(t0)) as f32 / (2.0 * dt);
                let mut area2 = sint + tint - 2.0 * sint * tint;
                if ds > 1.0 || dt > 1.0 {
                    area2 = 0.5
                };
                return self.tex1.evaluate(si) * (1.0 - area2) + self.tex2.evaluate(si) * area2;
            }
        }
    }
}
