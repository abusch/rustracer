use std::sync::Arc;

use bounds::Bounds3f;
use interaction::SurfaceInteraction;
use light::{Light, LightFlags};
use primitive::Primitive;
use ray::Ray;
use stats;

pub struct Scene {
    pub lights: Vec<Arc<Light + Sync + Send>>,
    pub infinite_lights: Vec<Arc<Light + Sync + Send>>,
    aggregate: Arc<Primitive + Sync + Send>,
}

impl Scene {
    pub fn new(
        aggregate: Arc<Primitive + Sync + Send>,
        lights: Vec<Arc<Light + Sync + Send>>,
    ) -> Scene {
        let mut scene = Scene {
            lights: Vec::new(),
            infinite_lights: Vec::new(),
            aggregate: aggregate,
        };

        let mut infinite_lights = Vec::new();

        for l in &lights {
            l.preprocess(&scene);
            if l.flags().contains(LightFlags::INFINITE) {
                infinite_lights.push(l.clone());
            }
        }

        ::std::mem::replace(&mut scene.lights, lights);
        ::std::mem::replace(&mut scene.infinite_lights, infinite_lights);

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
