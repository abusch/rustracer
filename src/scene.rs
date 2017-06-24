use std::sync::Arc;
use std::mem;

use bounds::Bounds3f;
use interaction::SurfaceInteraction;
use light::Light;
use primitive::Primitive;
use ray::Ray;

pub struct Scene {
    pub lights: Vec<Arc<Light + Sync + Send>>,
    aggregate: Arc<Primitive + Sync + Send>,
}

impl Scene {
    pub fn new(aggregate: Arc<Primitive + Sync + Send>,
               lights: Vec<Arc<Light + Sync + Send>>)
               -> Scene {
        let mut scene = Scene {
            lights: Vec::new(),
            aggregate: aggregate,
        };
        // TODO There's a bit of a circular reference with AreaLight <-> Shape <-> GeometricPrimitive which
        // doesn't play well with mutation needed by preprocessing...
        // for l in lights.iter_mut() {
        //     l.borrow_mut().preprocess(&scene);
        // }
        mem::replace(&mut scene.lights, lights);

        scene
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
