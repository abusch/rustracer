use std::ops::{Add, AddAssign, Sub, Div, Mul, Index, IndexMut};

use na;
use num::{Zero, One};

#[derive(Debug,Copy,PartialEq,Clone,Default)]
pub struct Spectrum {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Spectrum {
    pub fn rgb(r: f32, g: f32, b: f32) -> Spectrum {
        Spectrum { r: r, g: g, b: b }
    }

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

    pub fn to_srgb(&self) -> Spectrum {
        let a = 0.055f32;
        let b = 1f32 / 2.4;
        let mut srgb = Spectrum::black();
        for i in 0..3 {
            if self[i] <= 0.0031308 {
                srgb[i] = 12.92 * self[i];
            } else {
                srgb[i] = (1.0 + a) * f32::powf(self[i], b) - a;
            }
        }
        srgb
    }

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

    pub fn is_black(&self) -> bool {
        self.r == 0.0 && self.g == 0.0 && self.b == 0.0
    }

    pub fn has_nan(&self) -> bool {
        self.r.is_nan() || self.g.is_nan() || self.b.is_nan()
    }

    pub fn is_infinite(&self) -> bool {
        self.r.is_infinite() || self.g.is_infinite() || self.b.is_infinite()
    }

    pub fn sqrt(&self) -> Spectrum {
        Spectrum::rgb(self.r.sqrt(), self.g.sqrt(), self.b.sqrt())
    }
}

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

impl From<Spectrum> for [u8; 3] {
    fn from(c: Spectrum) -> [u8; 3] {
        [(na::clamp(c.r, 0.0, 1.0) * 255.0) as u8,
         (na::clamp(c.g, 0.0, 1.0) * 255.0) as u8,
         (na::clamp(c.b, 0.0, 1.0) * 255.0) as u8]
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
