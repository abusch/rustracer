use std::ops::{Add, Div, Mul, Sub};
use std::f32;
use std::mem;
use fp::Ieee754;

use super::MACHINE_EPSILON;

#[derive(Debug, Clone, Copy)]
pub struct EFloat {
    v: f32,
    low: f32,
    high: f32,
}

impl EFloat {
    pub fn new(v: f32, err: f32) -> EFloat {
        let (low, high) = if err == 0.0 {
            (v, v)
        } else {
            ((v - err).prev(), (v + err).next())
        };

        let r = EFloat {
            v: v,
            low: low,
            high: high,
        };
        r.check();
        r
    }

    pub fn lower_bound(&self) -> f32 {
        self.low
    }

    pub fn upper_bound(&self) -> f32 {
        self.high
    }

    pub fn absolute_error(&self) -> f32 {
        self.high - self.low
    }

    pub fn sqrt(&self) -> EFloat {
        let r = EFloat {
            v: self.v.sqrt(),
            low: self.lower_bound().sqrt().prev(),
            high: self.upper_bound().sqrt().next(),
        };
        r.check();
        r
    }

    pub fn abs(&self) -> EFloat {
        let r = if self.low >= 0.0 {
            // The entire interval is greater than 0: nothing to do!
            *self
        } else if self.high <= 0.0 {
            // The entire interval is less than zero: just inverse everything
            EFloat {
                v: -self.v,
                low: -self.high,
                high: -self.low,
            }
        } else {
            // The interval straddles zero
            EFloat {
                v: self.v.abs(),
                low: 0.0,
                high: (-self.low).max(self.high),
            }
        };
        r.check();
        r
    }

    #[inline]
    pub fn check(&self) {
        assert!(!self.v.is_nan());
        assert!(!self.low.is_nan());
        assert!(!self.high.is_nan());
        if !self.low.is_infinite() && !self.low.is_nan() && !self.high.is_infinite() &&
           !self.high.is_nan() {
            assert!(self.low <= self.high);
            assert!(self.low <= self.v);
            assert!(self.v <= self.high);
        }
    }
}

impl Default for EFloat {
    fn default() -> Self {
        EFloat::new(0.0, 0.0)
    }
}

// Quadratic solver
pub fn solve_quadratic(a: &EFloat, b: &EFloat, c: &EFloat) -> Option<(EFloat, EFloat)> {
    let discrim: f64 = f64::from(b.v) * f64::from(b.v) - 4f64 * f64::from(a.v) * f64::from(c.v);
    if discrim < 0.0 {
        return None;
    }

    let root_discrim = discrim.sqrt();
    let float_root_discrim = EFloat::new(root_discrim as f32,
                                         MACHINE_EPSILON * root_discrim as f32);

    let q = if b.v < 0.0 {
        -0.5 * (*b - float_root_discrim)
    } else {
        -0.5 * (*b + float_root_discrim)
    };
    let mut t0 = q / *a;
    let mut t1 = *c / q;
    if t0.v > t1.v {
        mem::swap(&mut t0, &mut t1);
    }

    Some((t0, t1))
}

impl PartialEq for EFloat {
    fn eq(&self, other: &Self) -> bool {
        self.v == other.v
    }
}

// Operator overloads

impl Add<EFloat> for EFloat {
    type Output = EFloat;

    fn add(self, f: EFloat) -> EFloat {
        let r = EFloat {
            v: self.v + f.v,
            low: (self.lower_bound() + f.lower_bound()).prev(),
            high: (self.upper_bound() + f.upper_bound()).next(),
        };
        r.check();
        r
    }
}

impl Sub<EFloat> for EFloat {
    type Output = EFloat;

    fn sub(self, f: EFloat) -> EFloat {
        let r = EFloat {
            v: self.v - f.v,
            low: (self.lower_bound() - f.upper_bound()).prev(),
            high: (self.upper_bound() - f.lower_bound()).next(),
        };
        r.check();
        r
    }
}

impl Mul<EFloat> for EFloat {
    type Output = EFloat;

    fn mul(self, f: EFloat) -> EFloat {
        let prod: [f32; 4] = [self.lower_bound() * f.lower_bound(),
                              self.upper_bound() * f.lower_bound(),
                              self.lower_bound() * f.upper_bound(),
                              self.upper_bound() * f.upper_bound()];

        let r = EFloat {
            v: self.v * f.v,
            low: f32::min(f32::min(prod[0], prod[1]), f32::min(prod[2], prod[3])).prev(),
            high: f32::max(f32::max(prod[0], prod[1]), f32::max(prod[2], prod[3])).next(),
        };
        r.check();
        r
    }
}

impl Div<EFloat> for EFloat {
    type Output = EFloat;

    fn div(self, f: EFloat) -> EFloat {
        let (low, high) = if f.lower_bound() < 0.0 && f.upper_bound() > 0.0 {
            (f32::NEG_INFINITY, f32::INFINITY)
        } else {
            let div: [f32; 4] = [self.lower_bound() / f.lower_bound(),
                                 self.upper_bound() / f.lower_bound(),
                                 self.lower_bound() / f.upper_bound(),
                                 self.upper_bound() / f.upper_bound()];
            (f32::min(f32::min(div[0], div[1]), f32::min(div[2], div[3])).prev(),
             f32::max(f32::max(div[0], div[1]), f32::max(div[2], div[3])).next())
        };
        let r = EFloat {
            v: self.v / f.v,
            low: low,
            high: high,
        };
        r.check();
        r
    }
}

impl From<f32> for EFloat {
    fn from(v: f32) -> EFloat {
        EFloat::new(v, 0.0)
    }
}

impl From<EFloat> for f32 {
    fn from(v: EFloat) -> f32 {
        v.v
    }
}

impl Add<f32> for EFloat {
    type Output = EFloat;
    fn add(self, f: f32) -> EFloat {
        self + EFloat::from(f)
    }
}

impl Sub<f32> for EFloat {
    type Output = EFloat;
    fn sub(self, f: f32) -> EFloat {
        self - EFloat::from(f)
    }
}

impl Mul<f32> for EFloat {
    type Output = EFloat;
    fn mul(self, f: f32) -> EFloat {
        self * EFloat::from(f)
    }
}

impl Div<f32> for EFloat {
    type Output = EFloat;
    fn div(self, f: f32) -> EFloat {
        self / EFloat::from(f)
    }
}

impl Add<EFloat> for f32 {
    type Output = EFloat;

    fn add(self, f: EFloat) -> EFloat {
        EFloat::from(self) + f
    }
}

impl Sub<EFloat> for f32 {
    type Output = EFloat;

    fn sub(self, f: EFloat) -> EFloat {
        EFloat::from(self) - f
    }
}

impl Mul<EFloat> for f32 {
    type Output = EFloat;

    fn mul(self, f: EFloat) -> EFloat {
        EFloat::from(self) * f
    }
}

impl Div<EFloat> for f32 {
    type Output = EFloat;

    fn div(self, f: EFloat) -> EFloat {
        EFloat::from(self) / f
    }
}
