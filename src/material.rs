use colour::Colourf;

pub struct Material {
    pub surface_colour: Colourf,
    pub emission_colour: Option<Colourf>,
    pub transparency: f32,
    pub reflection: f32,
}

impl Material {
    pub fn new(sc: Colourf, ec: Option<Colourf>, rf: f32, tr: f32) -> Material {
        Material {
            surface_colour: sc,
            emission_colour: ec,
            transparency: tr,
            reflection: rf
        }
    }
}
