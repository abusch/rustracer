use Point;
use colour::Colourf;

pub struct Light {
    pub pos: Point,
    pub emission_colour: Colourf,
}

impl Light {
    pub fn new(p: Point, ec: Colourf) -> Light {
        Light {pos: p, emission_colour: ec}
    }
}
