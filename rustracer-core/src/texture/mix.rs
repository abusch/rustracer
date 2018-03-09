use std::sync::Arc;
use std::fmt::Debug;
use std::ops::{Add, Mul};

use Transform;
use interaction::SurfaceInteraction;
use paramset::TextureParams;
use spectrum::Spectrum;
use texture::{Texture, TextureFloat};

#[derive(Debug)]
pub struct MixTexture<T> {
    tex1: Arc<dyn Texture<T>>,
    tex2: Arc<dyn Texture<T>>,
    amount: Arc<TextureFloat>,
}

impl<T> Texture<T> for MixTexture<T>
where
    T: Debug,
    T: Mul<f32, Output = T>,
    T: Add<Output = T>,
{
    fn evaluate(&self, si: &SurfaceInteraction) -> T {
        let t1 = self.tex1.evaluate(si);
        let t2 = self.tex2.evaluate(si);
        let amt = self.amount.evaluate(si);

        t1 * (1.0 - amt) + t2 * amt
    }
}

impl MixTexture<f32> {
    pub fn create_float(_tex2world: &Transform, tp: &TextureParams) -> MixTexture<f32> {
        MixTexture {
            tex1: tp.get_float_texture("tex1", 0.0),
            tex2: tp.get_float_texture("tex2", 1.0),
            amount: tp.get_float_texture("amount", 0.5),
        }
    }
}

impl MixTexture<Spectrum> {
    pub fn create_spectrum(_tex2world: &Transform, tp: &TextureParams) -> MixTexture<Spectrum> {
        MixTexture {
            tex1: tp.get_spectrum_texture("tex1", &Spectrum::black()),
            tex2: tp.get_spectrum_texture("tex2", &Spectrum::white()),
            amount: tp.get_float_texture("amount", 0.5),
        }
    }
}
