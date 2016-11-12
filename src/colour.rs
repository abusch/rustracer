use std::ops::{Add, AddAssign, Sub, Div, Mul, Index, IndexMut};
use na;

#[derive(Debug,Copy,PartialEq,Clone)]
pub struct Colourf {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Colourf {
    pub fn rgb(r: f32, g: f32, b: f32) -> Colourf {
        Colourf { r: r, g: g, b: b }
    }

    pub fn grey(v: f32) -> Colourf {
        Colourf { r: v, g: v, b: v }
    }

    pub fn white() -> Colourf {
        Colourf::rgb(1.0, 1.0, 1.0)
    }

    pub fn black() -> Colourf {
        Colourf::rgb(0.0, 0.0, 0.0)
    }

    pub fn to_srgb(&self) -> Colourf {
        let a = 0.055f32;
        let b = 1f32 / 2.4;
        let mut srgb = Colourf::black();
        for i in 0..3 {
            if self[i] <= 0.0031308 {
                srgb[i] = 12.92 * self[i];
            } else {
                srgb[i] = (1.0 + a) * f32::powf(self[i], b) - a;
            }
        }
        srgb
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

    pub fn sqrt(&self) -> Colourf {
        Colourf::rgb(self.r.sqrt(), self.g.sqrt(), self.b.sqrt())
    }
}

impl Add<Colourf> for Colourf {
    type Output = Colourf;

    fn add(self, rhs: Colourf) -> Colourf {
        Colourf {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl Sub<Colourf> for Colourf {
    type Output = Colourf;

    fn sub(self, rhs: Colourf) -> Colourf {
        Colourf {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
        }
    }
}

impl AddAssign<Colourf> for Colourf {
    fn add_assign(&mut self, rhs: Colourf) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

impl Mul<Colourf> for Colourf {
    type Output = Colourf;

    fn mul(self, rhs: Colourf) -> Colourf {
        Colourf::rgb(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b)
    }
}

impl Div<Colourf> for Colourf {
    type Output = Colourf;

    fn div(self, rhs: Colourf) -> Colourf {
        Colourf::rgb(self.r / rhs.r, self.g / rhs.g, self.b / rhs.b)
    }
}

impl Add<f32> for Colourf {
    type Output = Colourf;

    fn add(self, rhs: f32) -> Colourf {
        Colourf {
            r: self.r + rhs,
            g: self.g + rhs,
            b: self.b + rhs,
        }
    }
}

impl Sub<f32> for Colourf {
    type Output = Colourf;

    fn sub(self, rhs: f32) -> Colourf {
        Colourf {
            r: self.r - rhs,
            g: self.g - rhs,
            b: self.b - rhs,
        }
    }
}

impl Mul<f32> for Colourf {
    type Output = Colourf;

    fn mul(self, rhs: f32) -> Colourf {
        Colourf::rgb(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}

impl Mul<Colourf> for f32 {
    type Output = Colourf;

    fn mul(self, rhs: Colourf) -> Colourf {
        Colourf::rgb(self * rhs.r, self * rhs.g, self * rhs.b)
    }
}

impl Div<f32> for Colourf {
    type Output = Colourf;

    fn div(self, rhs: f32) -> Colourf {
        Colourf::rgb(self.r / rhs, self.g / rhs, self.b / rhs)
    }
}

impl From<Colourf> for [u8; 3] {
    fn from(c: Colourf) -> [u8; 3] {
        [(na::clamp(c.r, 0.0, 1.0) * 255.0) as u8,
         (na::clamp(c.g, 0.0, 1.0) * 255.0) as u8,
         (na::clamp(c.b, 0.0, 1.0) * 255.0) as u8]
    }
}

impl Index<usize> for Colourf {
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

impl IndexMut<usize> for Colourf {
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
