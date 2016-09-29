use filter::Filter;

pub struct MitchellNetravali {
    width: f32,
    height: f32,
    inv_width: f32,
    inv_height: f32,
    b: f32,
    c: f32,
}

impl MitchellNetravali {
    pub fn new(w: f32, h: f32, b: f32, c: f32) -> MitchellNetravali {
        MitchellNetravali {
            width: w,
            height: h,
            inv_width: 1.0 / w,
            inv_height: 1.0 / h,
            b: b,
            c: c,
        }
    }

    fn mitchell_1d(&self, x: f32) -> f32 {
        let fx = (x * 2.0).abs();
        if fx < 1.0 {
            ((12.0 - 9.0 * self.b - 6.0 * self.c) * fx * fx * fx +
             (-18.0 + 12.0 * self.b + 6.0 * self.c) * fx * fx + (6.0 - 2.0 * self.b)) *
            (1.0 / 6.0)
        } else if fx < 2.0 {
            ((-self.b - 6.0 * self.c) * fx * fx * fx + (6.0 * self.b + 30.0 * self.c) * fx * fx +
             (-12.0 * self.b - 48.0 * self.c) * fx + (8.0 * self.b + 24.0 * self.c)) *
            (1.0 / 6.0)
        } else {
            0.0
        }
    }
}

impl Filter for MitchellNetravali {
    fn evaluate(&self, x: f32, y: f32) -> f32 {
        self.mitchell_1d(x * self.inv_width) * self.mitchell_1d(y * self.inv_height)
    }

    fn width(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    fn inv_width(&self) -> (f32, f32) {
        (self.inv_width, self.inv_height)
    }
}
