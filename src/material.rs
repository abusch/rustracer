use colour::Colourf;

pub struct Material {
    pub surface_colour: Colourf,
    pub transparency: f32,
    pub reflection: f32,
}

impl Material {
    pub fn new(sc: Colourf, rf: f32, tr: f32) -> Material {
        Material {
            surface_colour: sc,
            transparency: tr,
            reflection: rf
        }
    }
}
