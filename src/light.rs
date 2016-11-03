use std::f32;
use std::f32::consts::*;
use na::Norm;

use Point;
use Vector;
use colour::Colourf;
use intersection::Intersection;

pub trait Light {
    /// Sample the light source for an outgoing direction wo.
    /// Return a triplet of:
    ///  * emitted light in the sampled direction
    ///  * the sampled direction wi
    ///  * the pdf for that direction
    fn sample_li(&self,
                 isect: &Intersection,
                 wo: Vector,
                 sample: (f32, f32))
                 -> (Colourf, Vector, f32);
}

#[derive(Debug)]
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
    fn sample_li(&self,
                 isect: &Intersection,
                 wo: Vector,
                 sample: (f32, f32))
                 -> (Colourf, Vector, f32) {
        let wi = isect.dg.phit - self.pos;
        let r2 = wi.norm_squared();
        let l_i = self.emission_colour / (4.0 * PI * r2);

        (l_i, wi.normalize(), 1.0)
    }
}

#[derive(Debug)]
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
    fn sample_li(&self,
                 isect: &Intersection,
                 wo: Vector,
                 sample: (f32, f32))
                 -> (Colourf, Vector, f32) {
        (self.emission_colour, self.dir, 1.0)
    }
}
