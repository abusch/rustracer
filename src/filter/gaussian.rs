use Vector2f;
use paramset::ParamSet;
use super::Filter;

pub struct GaussianFilter {
    radius: (f32, f32),
    inv_radius: (f32, f32),
    alpha: f32,
    expx: f32,
    expy: f32,
}

impl GaussianFilter {
    pub fn new(radius: &Vector2f, alpha: f32) -> GaussianFilter {
        GaussianFilter {
            radius: (radius.x, radius.y),
            inv_radius: (1.0 / radius.x, 1.0 / radius.y),
            alpha: alpha,
            expx: (-alpha * radius.x * radius.x).exp(),
            expy: (-alpha * radius.y * radius.y).exp(),
        }
    }

    fn gaussian(&self, d: f32, expv: f32) -> f32 {
        ((-self.alpha * d * d).exp() - expv).max(0f32)
    }

    pub fn create(ps: &mut ParamSet) -> Box<Filter + Send + Sync> {
        let xw = ps.find_one_float("xwidth", 2.0);
        let yw = ps.find_one_float("ywidth", 2.0);
        let alpha = ps.find_one_float("alpha", 2.0);
        Box::new(GaussianFilter::new(&Vector2f::new(xw, yw), alpha))
    }
}

impl Filter for GaussianFilter {
    fn evaluate(&self, x: f32, y: f32) -> f32 {
        self.gaussian(x, self.expx) * self.gaussian(y, self.expy)
    }

    fn width(&self) -> (f32, f32) {
        self.radius
    }

    fn inv_width(&self) -> (f32, f32) {
        self.inv_radius
    }
}
