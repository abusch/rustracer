use std::convert::From;
use std::f32;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Index, IndexMut, Mul, MulAssign, Sub};

use num::{One, Zero};

use crate::cie;
use crate::{clamp, find_interval, lerp};

/// Represents a linear RGB spectrum.
/// TODO Rename this to `RGBSpectrum` and make `Spectrum` a type alias to this so we can also support
/// full spectral rendering.
#[derive(Debug, Copy, PartialEq, Clone, Default)]
pub struct Spectrum {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Spectrum {
    /// Create an RGB spectrum from its components
    pub fn rgb(r: f32, g: f32, b: f32) -> Spectrum {
        Spectrum { r, g, b }
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
    pub fn to_srgb(self) -> [u8; 3] {
        let a = 0.055f32;
        let b = 1f32 / 2.4;
        let mut srgb = [0; 3];
        for i in 0..3 {
            let v = if self[i] <= 0.0031308 {
                12.92 * self[i]
            } else {
                (1.0 + a) * f32::powf(self[i], b) - a
            };
            srgb[i] = clamp(v * 255.0 + 0.5, 0.0, 255.0) as u8;
        }
        srgb
    }

    /// Convert a non-linear sRGB value to a linear RGB spectrum.
    pub fn from_srgb(rgb: [u8; 3]) -> Spectrum {
        fn as_float(v: u8) -> f32 {
            f32::from(v) / 255.0
        }

        Spectrum::rgb(
            inverse_gamma_convert_float(as_float(rgb[0])),
            inverse_gamma_convert_float(as_float(rgb[1])),
            inverse_gamma_convert_float(as_float(rgb[2])),
        )
    }

    pub fn inverse_gamma_correct(&self) -> Spectrum {
        Spectrum::rgb(
            inverse_gamma_convert_float(self.r),
            inverse_gamma_convert_float(self.g),
            inverse_gamma_convert_float(self.b),
        )
    }

    /// Convert a linear spectrum in XYZ format to a linear RGB format.
    pub fn from_xyz(xyz: &[f32; 3]) -> Spectrum {
        let r = 3.240479 * xyz[0] - 1.537150 * xyz[1] - 0.498535 * xyz[2];
        let g = -0.969256 * xyz[0] + 1.875991 * xyz[1] + 0.041556 * xyz[2];
        let b = 0.055648 * xyz[0] - 0.204043 * xyz[1] + 1.057311 * xyz[2];
        Spectrum::rgb(r, g, b)
    }

    pub fn to_xyz(self) -> [f32; 3] {
        let mut xyz = [0.0, 0.0, 0.0];

        xyz[0] = 0.412453 * self.r + 0.357580 * self.g + 0.180423 * self.b;
        xyz[1] = 0.212671 * self.r + 0.715160 * self.g + 0.072169 * self.b;
        xyz[2] = 0.019334 * self.r + 0.119193 * self.g + 0.950227 * self.b;

        xyz
    }

    /// Create a spectrum from a series of (wavelength, value) samples from an SPD (Spectral Power
    /// Distribution).
    pub fn from_sampled(lambda: &[f32], v: &[f32], n: usize) -> Spectrum {
        // TODO sort by wavelength if needed
        let mut xyz = [0.0; 3];
        for i in 0..cie::N_CIE_SAMPLES {
            let val = interpolate_spectrum_samples(lambda, v, n, cie::CIE_LAMBDA[i]);
            xyz[0] += val * cie::CIE_X[i];
            xyz[1] += val * cie::CIE_Y[i];
            xyz[2] += val * cie::CIE_Z[i];
        }
        let scale = (cie::CIE_LAMBDA[cie::N_CIE_SAMPLES - 1] - cie::CIE_LAMBDA[0])
            / (cie::CIE_Y_INTEGRAL * cie::N_CIE_SAMPLES as f32);
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

    /// Return the luminance of the Spectrum
    pub fn y(&self) -> f32 {
        let y_height: [f32; 3] = [0.212671, 0.715160, 0.072169];
        y_height[0] * self[0] + y_height[1] * self[1] + y_height[2] * self[2]
    }

    pub fn max_component_value(&self) -> f32 {
        self.r.max(self.g).max(self.b)
    }

    pub fn clamp(&self) -> Spectrum {
        Spectrum::rgb(
            clamp(self.r, 0.0, f32::INFINITY),
            clamp(self.g, 0.0, f32::INFINITY),
            clamp(self.b, 0.0, f32::INFINITY),
        )
    }
}

#[allow(non_upper_case_globals)]
pub fn blackbody(lambda: &[f32], temp: f32) -> Vec<f32> {
    assert!(temp > 0.0);

    const c: f32 = 299792458.0;
    const h: f32 = 6.62606957e-34;
    const kb: f32 = 1.3806488e-23;
    lambda
        .iter()
        .map(|lambda_i| {
            // Compute emitted radiance for blackbody at wavelength _lambda[i]_
            let l = lambda_i * 1e-9;
            let lambda5 = (l * l) * (l * l) * l;
            let Le_i = (2.0 * h * c * c) / (lambda5 * (f32::exp((h * c) / (l * kb * temp)) - 1.0));
            assert!(!Le_i.is_infinite());
            Le_i
        })
        .collect()
}

pub fn blackbody_normalized(lambda: &[f32], temp: f32) -> Vec<f32> {
    let Le = blackbody(lambda, temp);
    // normalize Le values based on maximum blackbody radiance
    let lambda_max = 2.8977721e-3 / temp * 1e9;
    let max_L = blackbody(&[lambda_max], temp);

    Le.iter().map(|v| v / max_L[0]).collect()
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

impl MulAssign<f32> for Spectrum {
    fn mul_assign(&mut self, v: f32) {
        self.r *= v;
        self.g *= v;
        self.b *= v;
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

impl fmt::Display for Spectrum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ {}, {}, {} ]", self.r, self.g, self.b)
    }
}

impl From<f32> for Spectrum {
    fn from(v: f32) -> Spectrum {
        Spectrum::grey(v)
    }
}

pub fn inverse_gamma_convert_float(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) * 1.0 / 1.055).powf(2.4)
    }
}

pub fn gamma_correct(v: f32) -> f32 {
    if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * f32::powf(v, 1.0 / 2.4) - 0.055
    }
}
