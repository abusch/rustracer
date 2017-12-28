use std::sync::Arc;

use bounds::Bounds3f;
use interaction::SurfaceInteraction;
use light::{Light, LightFlags};
use primitive::Primitive;
use ray::Ray;

stat_counter!("Intersections/Regular ray intersection tests",
              n_intersection_tests);
stat_counter!("Intersections/Shadow ray intersection tests",
              n_shadow_tests);
pub fn init_stats() {
    n_intersection_tests::init();
    n_shadow_tests::init();
}

pub struct Scene {
    pub lights: Vec<Arc<Light>>,
    pub infinite_lights: Vec<Arc<Light>>,
    aggregate: Arc<Primitive>,
}

impl Scene {
    pub fn new(aggregate: Arc<Primitive>,
               lights: Vec<Arc<Light>>)
               -> Scene {
        let mut scene = Scene {
            lights: Vec::new(),
            infinite_lights: Vec::new(),
            aggregate: aggregate,
        };

        let mut infinite_lights = Vec::new();

        for l in &lights {
            l.preprocess(&scene);
            if l.flags().contains(LightFlags::INFINITE) {
                infinite_lights.push(Arc::clone(&l));
            }
        }

        ::std::mem::replace(&mut scene.lights, lights);
        ::std::mem::replace(&mut scene.infinite_lights, infinite_lights);

        scene
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction> {
        n_intersection_tests::inc();
        self.aggregate.intersect(ray)
    }

    pub fn intersect_p(&self, ray: &Ray) -> bool {
        n_shadow_tests::inc();
        self.aggregate.intersect_p(ray)
    }

    pub fn world_bounds(&self) -> Bounds3f {
        self.aggregate.world_bounds()
    }
}
