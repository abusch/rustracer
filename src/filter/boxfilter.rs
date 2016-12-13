use filter::Filter;

pub struct BoxFilter {}

impl Filter for BoxFilter {
    fn evaluate(&self, _x: f32, _y: f32) -> f32 {
        1.0
    }

    fn width(&self) -> (f32, f32) {
        (0.5, 0.5)
    }

    fn inv_width(&self) -> (f32, f32) {
        (2.0, 2.0)
    }
}
