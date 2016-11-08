use std::sync::Arc;
use ray::Ray;
use bounds::Bounds3f;
use shapes::{Shape, SurfaceInteraction};


pub trait AreaLight {}

pub trait Material {}

pub trait Primitive {
    fn world_bounds(&self) -> Bounds3f;

    fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction>;

    fn intersect_p(&self, ray: &mut Ray) -> bool;

    fn area_light(&self) -> Option<Box<AreaLight>>;

    fn material(&self) -> Option<Box<Material>>;
}

pub struct GeometricPrimitive {
    shape: Arc<Shape>,
}

impl Primitive for GeometricPrimitive {
    fn world_bounds(&self) -> Bounds3f {
        self.shape.world_bounds()
    }

    fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction> {
        unimplemented!()
    }

    fn intersect_p(&self, ray: &mut Ray) -> bool {
        unimplemented!()
    }

    fn area_light(&self) -> Option<Box<AreaLight>> {
        unimplemented!()
    }

    fn material(&self) -> Option<Box<Material>> {
        unimplemented!()
    }
}
