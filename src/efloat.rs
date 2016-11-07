use std::ops::{Add, Mul, Sub, Div};
use std::f32;
use fp::Ieee754;

use super::gamma;

struct EFloat {
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

        EFloat {
            v: v,
            low: low,
            high: high,
        }
    }

    pub fn lower_bound(&self) -> f32 {
        self.low
    }

    pub fn upper_bound(&self) -> f32 {
        self.high
    }
}

impl Default for EFloat {
    fn default() -> Self {
        EFloat::new(0.0, 0.0)
    }
}

impl Add<EFloat> for EFloat {
    type Output = EFloat;

    fn add(self, f: EFloat) -> EFloat {
        EFloat {
            v: self.v + f.v,
            low: (self.lower_bound() + f.lower_bound()).prev(),
            high: (self.upper_bound() + f.upper_bound()).next(),
        }
    }
}

impl Sub<EFloat> for EFloat {
    type Output = EFloat;

    fn sub(self, f: EFloat) -> EFloat {
        EFloat {
            v: self.v - f.v,
            low: (self.lower_bound() - f.lower_bound()).prev(),
            high: (self.upper_bound() - f.upper_bound()).next(),
        }
    }
}

impl Mul<EFloat> for EFloat {
    type Output = EFloat;

    fn mul(self, f: EFloat) -> EFloat {
        let prod: [f32; 4] = [self.lower_bound() * f.lower_bound(),
                              self.upper_bound() * f.lower_bound(),
                              self.lower_bound() * f.upper_bound(),
                              self.upper_bound() * f.upper_bound()];

        EFloat {
            v: self.v * f.v,
            low: f32::min(f32::min(prod[0], prod[1]), f32::min(prod[2], prod[3])).prev(),
            high: f32::max(f32::max(prod[0], prod[1]), f32::max(prod[2], prod[3])).next(),
        }
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
        EFloat {
            v: self.v / f.v,
            low: low,
            high: high,
        }
    }
}

impl From<f32> for EFloat {
    fn from(v: f32) -> EFloat {
        EFloat::new(v, 0.0)
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
