use Vector2f;
use filter::Filter;
use paramset::ParamSet;

#[derive(Debug)]
pub struct TriangleFilter {
    radius: Vector2f,
    inv_radius: Vector2f,
}

impl TriangleFilter {
    pub fn new(radius_x: f32, radius_y: f32) -> TriangleFilter {
        TriangleFilter {
            radius: Vector2f::new(radius_x, radius_y),
            inv_radius: Vector2f::new(1.0 / radius_x, 1.0 / radius_y),
        }
    }

    pub fn create(ps: &mut ParamSet) -> Box<Filter> {
        let xw = ps.find_one_float("xwidth", 2.0);
        let yw = ps.find_one_float("ywidth", 2.0);

        Box::new(TriangleFilter::new(xw, yw))
    }
}

impl Filter for TriangleFilter {
    fn evaluate(&self, x: f32, y: f32) -> f32 {
        f32::max(0.0, self.radius.x - f32::abs(x)) * f32::max(0.0, self.radius.y - f32::abs(y))
    }

    fn width(&self) -> (f32, f32) {
        (self.radius.x, self.radius.y)
    }

    fn inv_width(&self) -> (f32, f32) {
        (self.inv_radius.x, self.inv_radius.y)
    }
}
