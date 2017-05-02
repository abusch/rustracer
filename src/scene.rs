use std::sync::Arc;
use std::mem;

use bounds::Bounds3f;
use interaction::SurfaceInteraction;
use light::Light;
use primitive::Primitive;
use ray::Ray;

pub struct Scene {
    pub lights: Vec<Arc<Light + Sync + Send>>,
    pub primitives: Vec<Box<Primitive + Sync + Send>>,
}

impl Scene {
    pub fn new(primitives: Vec<Box<Primitive + Sync + Send>>,
               lights: Vec<Arc<Light + Sync + Send>>)
               -> Scene {
        let mut scene = Scene {
            lights: Vec::new(),
            primitives: primitives,
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
        self.primitives
            .iter()
            .fold(None, |r, p| p.intersect(ray).or(r))
    }

    pub fn intersect_p(&self, ray: &Ray) -> bool {
        self.primitives
            .iter()
            .fold(false, |r, p| p.intersect_p(ray) || r)
    }

    pub fn world_bounds(&self) -> Bounds3f {
        self.primitives
            .iter()
            .fold(Bounds3f::new(),
                  |r, p| Bounds3f::union(&r, &p.world_bounds()))
    }
}
