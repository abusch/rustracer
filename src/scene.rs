use na::Norm;

use ::Vector;
use bvh::BVH;
use camera::Camera;
use instance::Instance;
use integrator::Integrator;
use intersection::Intersection;
use light::Light;
use ray::Ray;
use skydome::Atmosphere;

pub struct Scene {
    pub camera: Camera,
    bvh: BVH<Instance>,
    pub lights: Vec<Box<Light + Sync + Send>>,
    pub atmosphere: Atmosphere,
    pub integrator: Box<Integrator + Sync + Send>,
}

impl Scene {
    pub fn new(camera: Camera,
               integrator: Box<Integrator + Sync + Send>,
               objects: &mut Vec<Instance>,
               lights: Vec<Box<Light + Sync + Send>>)
               -> Scene {
        let bvh = BVH::new(16, objects);
        Scene {
            camera: camera,
            // objects: objects,
            bvh: bvh,
            lights: lights,
            atmosphere: Atmosphere::earth((Vector::y()).normalize()),
            integrator: integrator,
        }
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        self.bvh.intersect(ray, |r, i| i.intersect(r))
    }

    pub fn intersect_p(&self, ray: &mut Ray) -> bool {
        self.intersect(ray).is_some()
    }
}
