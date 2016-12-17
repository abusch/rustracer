use std::ops::{Add, AddAssign, Sub, Div, Mul, Index, IndexMut};

use na;
use num::{Zero, One};

use ::lerp;
use cie;

/// Represents a linear RGB spectrum.
/// TODO Rename this to `RGBSpectrum` and make `Spectrum` a type alias to this so we can also support
/// full spectral rendering.
#[derive(Debug,Copy,PartialEq,Clone,Default)]
pub struct Spectrum {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Spectrum {
    /// Create an RGB spectrum from its components
    pub fn rgb(r: f32, g: f32, b: f32) -> Spectrum {
        Spectrum { r: r, g: g, b: b }
    }

    /// Create an RGB spectrum where all the components have the same value.
    pub fn grey(v: f32) -> Spectrum {
        Spectrum { r: v, g: v, b: v }
    }

    pub fn white() -> Spectrum {
        Spectrum::rgb(1.0, 1.0, 1.0)
    }

    pub fn black() -> Spectrum {
        Spectrum::rgb(0.0, 0.0, 0.0)
    }

    pub fn red() -> Spectrum {
        Spectrum::rgb(1.0, 0.0, 0.0)
    }

    pub fn green() -> Spectrum {
        Spectrum::rgb(0.0, 1.0, 0.0)
    }

    pub fn blue() -> Spectrum {
        Spectrum::rgb(0.0, 0.0, 1.0)
    }

    /// Convert this linear RGB spectrum to non-linear sRGB and return the result as an array of
    /// bytes.
    pub fn to_srgb(&self) -> [u8; 3] {
        let a = 0.055f32;
        let b = 1f32 / 2.4;
        let mut srgb = [0; 3];
        for i in 0..3 {
            let v = if self[i] <= 0.0031308 {
                12.92 * self[i]
            } else {
                (1.0 + a) * f32::powf(self[i], b) - a
            };
            srgb[i] = na::clamp(v * 255.0, 0.0, 255.0) as u8;
        }
        srgb
    }

    /// Convert a non-linear sRGB value to a linear RGB spectrum.
    pub fn from_srgb(rgb: &[u8; 3]) -> Spectrum {
        fn convert(v: u8) -> f32 {
            let value = v as f32 / 255.0;
            if value <= 0.0031308 {
                12.92 * value
            } else {
                1.055 * value.powf(1.0 / 2.4) - 0.055
            }
        }

        Spectrum::rgb(convert(rgb[0]), convert(rgb[1]), convert(rgb[2]))
    }

    /// Convert a linear spectrum in XYZ format to a linear RGB format.
    pub fn from_xyz(xyz: &[f32; 3]) -> Spectrum {
        let r = 3.240479 * xyz[0] - 1.537150 * xyz[1] - 0.498535 * xyz[2];
        let g = -0.969256 * xyz[0] + 1.875991 * xyz[1] + 0.041556 * xyz[2];
        let b = 0.055648 * xyz[0] - 0.204043 * xyz[1] + 1.057311 * xyz[2];
        Spectrum::rgb(r, g, b)
    }

    /// Create a spectrum from a series of (wavelength, value) samples from an SPD (Spectral Power
    /// Distribution).
    pub fn from_sampled(lambda: &[f32], v: &[f32], n: usize) -> Spectrum {
        // TODO sort by wavelength if needed
        let mut xyz = [0.0; 3];
        for i in 0..cie::nCIESamples {
            let val = interpolate_spectrum_samples(lambda, v, n, cie::CIE_lambda[i]);
            xyz[0] += val * cie::CIE_X[i];
            xyz[1] += val * cie::CIE_Y[i];
            xyz[2] += val * cie::CIE_Z[i];
        }
        let scale = (cie::CIE_lambda[cie::nCIESamples - 1] - cie::CIE_lambda[0]) /
                    (cie::CIE_Y_integral * cie::nCIESamples as f32);
        xyz[0] *= scale;
        xyz[1] *= scale;
        xyz[2] *= scale;

        Self::from_xyz(&xyz)
    }

    /// Return true if the colour is black i.e (0, 0 ,0).
    pub fn is_black(&self) -> bool {
        self.r == 0.0 && self.g == 0.0 && self.b == 0.0
    }

    /// Return true if any of the components is NaN. Useful for debugging.
    pub fn has_nan(&self) -> bool {
        self.r.is_nan() || self.g.is_nan() || self.b.is_nan()
    }

    /// Return true if any of the components is infinite. Useful for debugging.
    pub fn is_infinite(&self) -> bool {
        self.r.is_infinite() || self.g.is_infinite() || self.b.is_infinite()
    }

    /// Return a spectrum where each component is the square root of the original component.
    pub fn sqrt(&self) -> Spectrum {
        Spectrum::rgb(self.r.sqrt(), self.g.sqrt(), self.b.sqrt())
    }
}

fn interpolate_spectrum_samples(lambda: &[f32], vals: &[f32], n: usize, l: f32) -> f32 {
    for i in 0..n - 1 {
        assert!(lambda[i + 1] > lambda[i]);
    }
    if l <= lambda[0] {
        return vals[0];
    }
    if l >= lambda[n - 1] {
        return vals[n - 1];
    }
    let offset = find_interval(n, |index| lambda[index] <= l);
    assert!(l >= lambda[offset] && l <= lambda[offset + 1]);
    let t = (l - lambda[offset]) / (lambda[offset + 1] - lambda[offset]);
    lerp(t, vals[offset], vals[offset + 1])
}

fn find_interval<P>(size: usize, pred: P) -> usize
    where P: Fn(usize) -> bool
{
    let mut first = 0;
    let mut len = size;
    while len > 0 {
        let half = len >> 1;
        let middle = first + half;
        // Bisect range based on value of _pred_ at _middle_
        if pred(middle) {
            first = middle + 1;
            len -= half + 1;
        } else {
            len = half;
        }
    }
    na::clamp(first - 1, 0, size - 2)
}


// Operators

impl Add<Spectrum> for Spectrum {
    type Output = Spectrum;

    fn add(self, rhs: Spectrum) -> Spectrum {
        Spectrum {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl Sub<Spectrum> for Spectrum {
    type Output = Spectrum;

    fn sub(self, rhs: Spectrum) -> Spectrum {
        Spectrum {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
        }
    }
}

impl AddAssign<Spectrum> for Spectrum {
    fn add_assign(&mut self, rhs: Spectrum) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

impl Mul<Spectrum> for Spectrum {
    type Output = Spectrum;

    fn mul(self, rhs: Spectrum) -> Spectrum {
        Spectrum::rgb(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b)
    }
}

impl Div<Spectrum> for Spectrum {
    type Output = Spectrum;

    fn div(self, rhs: Spectrum) -> Spectrum {
        Spectrum::rgb(self.r / rhs.r, self.g / rhs.g, self.b / rhs.b)
    }
}

impl Add<f32> for Spectrum {
    type Output = Spectrum;

    fn add(self, rhs: f32) -> Spectrum {
        Spectrum {
            r: self.r + rhs,
            g: self.g + rhs,
            b: self.b + rhs,
        }
    }
}

impl Sub<f32> for Spectrum {
    type Output = Spectrum;

    fn sub(self, rhs: f32) -> Spectrum {
        Spectrum {
            r: self.r - rhs,
            g: self.g - rhs,
            b: self.b - rhs,
        }
    }
}

impl Mul<f32> for Spectrum {
    type Output = Spectrum;

    fn mul(self, rhs: f32) -> Spectrum {
        Spectrum::rgb(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}

impl Mul<Spectrum> for f32 {
    type Output = Spectrum;

    fn mul(self, rhs: Spectrum) -> Spectrum {
        Spectrum::rgb(self * rhs.r, self * rhs.g, self * rhs.b)
    }
}

impl Div<f32> for Spectrum {
    type Output = Spectrum;

    fn div(self, rhs: f32) -> Spectrum {
        Spectrum::rgb(self.r / rhs, self.g / rhs, self.b / rhs)
    }
}

impl Index<usize> for Spectrum {
    type Output = f32;
    /// Access the channels by index
    ///
    /// - 0 = r
    /// - 1 = g
    /// - 2 = b
    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.r,
            1 => &self.g,
            2 => &self.b,
            _ => panic!("Invalid index into color"),
        }
    }
}

impl IndexMut<usize> for Spectrum {
    /// Access the channels by index
    ///
    /// - 0 = r
    /// - 1 = g
    /// - 2 = b
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i {
            0 => &mut self.r,
            1 => &mut self.g,
            2 => &mut self.b,
            _ => panic!("Invalid index into color"),
        }
    }
}

impl Zero for Spectrum {
    fn zero() -> Spectrum {
        Spectrum::black()
    }

    fn is_zero(&self) -> bool {
        self.is_black()
    }
}

impl One for Spectrum {
    fn one() -> Spectrum {
        Spectrum::white()
    }
}
