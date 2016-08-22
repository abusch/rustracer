use std::ops::{Add, AddAssign, Div, Mul, Index, IndexMut};

#[derive(Debug,Copy,PartialEq,Clone)]
pub struct Colourf {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

impl Colourf {
    pub fn rgb(r: f32, g: f32, b: f32) -> Colourf {
        Colourf {r: r, g: g, b: b, a: 0.0}
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

}

impl Add<Colourf> for Colourf {
    type Output = Colourf;

    fn add(self, rhs: Colourf) -> Colourf {
        Colourf {r: self.r + rhs.r, g: self.g + rhs.g, b: self.b + rhs.b, a: self.a + rhs.a}
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

impl Mul<f32> for Colourf {
    type Output = Colourf;

    fn mul(self, rhs: f32) -> Colourf {
        Colourf::rgb(self.r * rhs, self.g * rhs, self.b * rhs)
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
        [
            (f32::min(1.0, c.r) * 255.0) as u8,
            (f32::min(1.0, c.g) * 255.0) as u8,
            (f32::min(1.0, c.b) * 255.0) as u8
        ]
    }
}

impl Index<usize> for Colourf {
    type Output = f32;
    /// Access the channels by index
    /// 
    /// - 0 = r
    /// - 1 = g
    /// - 2 = b
    /// - 3 = a
    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.r,
            1 => &self.g,
            2 => &self.b,
            3 => &self.a,
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
    /// - 3 = a
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i {
            0 => &mut self.r,
            1 => &mut self.g,
            2 => &mut self.b,
            3 => &mut self.a,
            _ => panic!("Invalid index into color"),
        }
    }
}


