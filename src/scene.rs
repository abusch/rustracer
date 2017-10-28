use std::sync::Arc;
use std::mem;

use bounds::Bounds3f;
use interaction::SurfaceInteraction;
use light::{Light, LightFlags};
use primitive::Primitive;
use ray::Ray;

pub struct Scene {
    pub lights: Vec<Arc<Light + Sync + Send>>,
    pub infinite_lights: Vec<Arc<Light + Sync + Send>>,
    aggregate: Arc<Primitive + Sync + Send>,
}

impl Scene {
    pub fn new(aggregate: Arc<Primitive + Sync + Send>,
               lights: Vec<Arc<Light + Sync + Send>>)
               -> Scene {
        // TODO There's a bit of a circular reference with AreaLight <-> Shape <-> GeometricPrimitive which
        // doesn't play well with mutation needed by preprocessing...
        // for l in lights.iter_mut() {
        //     l.borrow_mut().preprocess(&scene);
        // }
        let mut infinite_lights = Vec::new();
        for l in &lights {
            if l.flags().contains(LightFlags::INFINITE) {
                infinite_lights.push(l.clone());
            }
        }

        Scene {
            lights: lights,
            infinite_lights: infinite_lights,
            aggregate: aggregate,
        }
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction> {
        self.aggregate.intersect(ray)
    }

    pub fn intersect_p(&self, ray: &Ray) -> bool {
        self.aggregate.intersect_p(ray)
    }

    pub fn world_bounds(&self) -> Bounds3f {
        self.aggregate.world_bounds()
    }
}
