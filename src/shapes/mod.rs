use ::Point;
use ray::Ray;

pub mod sphere;

pub struct SurfaceInteraction {
}

pub struct Bounds3f {
    p_min: Point,
    p_max: Point,
}

pub trait Shape {
    fn intersect(&self, ray: &Ray) -> Option<SurfaceInteraction>;

    fn intersect_p(&self, ray: &Ray) -> bool {
        self.intersect(ray).is_some()
    }

    fn area(&self) -> f32 {
        0.0
    }

    fn object_bounds(&self) -> Bounds3f;
}
