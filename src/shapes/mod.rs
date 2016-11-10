use std::sync::Arc;

use ::{Vector, Point, Point2f};
use bsdf::BSDF;
use ray::Ray;
use bounds::Bounds3f;
use primitive::Primitive;
use interaction::SurfaceInteraction;
use na::{self, Cross, Norm};

pub mod sphere;

pub trait Shape {
    fn intersect(&self, ray: &Ray) -> Option<(SurfaceInteraction, f32)>;

    fn intersect_p(&self, ray: &Ray) -> bool {
        self.intersect(ray).is_some()
    }

    fn area(&self) -> f32 {
        0.0
    }

    fn object_bounds(&self) -> Bounds3f;

    fn world_bounds(&self) -> Bounds3f;
}
