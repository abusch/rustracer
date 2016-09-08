use std::f32;
use std::f32::consts::*;
use na::Norm;

use Point;
use Vector;
use colour::Colourf;

pub trait Light {
    fn shading_info(&self, p: &Point) -> ShadingInfo;
}

pub struct PointLight {
    pub pos: Point,
    pub emission_colour: Colourf,
}

impl PointLight {
    pub fn new(p: Point, ec: Colourf) -> PointLight {
        PointLight {
            pos: p,
            emission_colour: ec,
        }
    }
}

impl Light for PointLight {
    fn shading_info(&self, p: &Point) -> ShadingInfo {
        let light_dir = self.pos - *p;
        let r2 = light_dir.norm_squared();
        let w_i = light_dir.normalize();
        let l_i = self.emission_colour / (4.0 * PI * r2);

        ShadingInfo {
            l_i: l_i,
            w_i: w_i,
            light_distance: r2.sqrt(),
        }
    }
}

pub struct DistantLight {
    pub dir: Vector,
    pub emission_colour: Colourf,
}

impl DistantLight {
    pub fn new(dir: Vector, ec: Colourf) -> DistantLight {
        DistantLight {
            dir: dir.normalize(),
            emission_colour: ec,
        }
    }
}

impl Light for DistantLight {
    fn shading_info(&self, _: &Point) -> ShadingInfo {
        ShadingInfo {
            l_i: self.emission_colour,
            w_i: -self.dir,
            light_distance: f32::INFINITY,
        }
    }
}

#[derive(Debug)]
pub struct ShadingInfo {
    pub l_i: Colourf,
    pub w_i: Vector,
    pub light_distance: f32,
}
