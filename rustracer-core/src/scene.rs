use std::sync::Arc;

use crate::bounds::Bounds3f;
use crate::interaction::SurfaceInteraction;
use crate::light::{Light, LightFlags};
use crate::primitive::Primitive;
use crate::ray::Ray;

stat_counter!(
    "Intersections/Regular ray intersection tests",
    n_intersection_tests
);
stat_counter!(
    "Intersections/Shadow ray intersection tests",
    n_shadow_tests
);
pub fn init_stats() {
    n_intersection_tests::init();
    n_shadow_tests::init();
}

pub struct Scene {
    pub lights: Vec<Arc<dyn Light>>,
    pub infinite_lights: Vec<Arc<dyn Light>>,
    aggregate: Arc<dyn Primitive>,
}

impl Scene {
    pub fn new(aggregate: Arc<dyn Primitive>, lights: Vec<Arc<dyn Light>>) -> Scene {
        let mut scene = Scene {
            lights: Vec::new(),
            infinite_lights: Vec::new(),
            aggregate,
        };

        let mut infinite_lights = Vec::new();

        for l in &lights {
            l.preprocess(&scene);
            if l.flags().contains(LightFlags::INFINITE) {
                infinite_lights.push(Arc::clone(l));
            }
        }

        scene.lights = lights;
        scene.infinite_lights = infinite_lights;

        scene
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction<'_, '_>> {
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
