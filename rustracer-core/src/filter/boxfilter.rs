use filter::Filter;
use paramset::ParamSet;

pub struct BoxFilter {
    radius: (f32, f32),
    inv_radius: (f32, f32),
}

impl BoxFilter {
    pub fn new(xwidth: f32, ywidth: f32) -> BoxFilter {
        BoxFilter {
            radius: (xwidth, ywidth),
            inv_radius: (1.0 / xwidth, 1.0 / ywidth),
        }
    }

    pub fn create(ps: &mut ParamSet) -> Box<Filter + Send + Sync> {
        let xw = ps.find_one_float("xwidth", 0.5);
        let yw = ps.find_one_float("ywidth", 0.5);

        Box::new(Self::new(xw, yw))
    }
}

impl Filter for BoxFilter {
    fn evaluate(&self, _x: f32, _y: f32) -> f32 {
        1.0
    }

    fn width(&self) -> (f32, f32) {
        self.radius
    }

    fn inv_width(&self) -> (f32, f32) {
        self.inv_radius
    }
}
