use na::Norm;

use ::Vector;
use camera::Camera;
use instance::Instance;
use integrator::Integrator;
use intersection::Intersection;
use light::Light;
use ray::Ray;
use skydome::Atmosphere;

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Instance>,
    pub lights: Vec<Box<Light + Sync + Send>>,
    pub atmosphere: Atmosphere,
    pub integrator: Box<Integrator + Sync + Send>,
}

impl Scene {
    pub fn new(camera: Camera,
               integrator: Box<Integrator + Sync + Send>,
               objects: Vec<Instance>,
               lights: Vec<Box<Light + Sync + Send>>)
               -> Scene {
        Scene {
            camera: camera,
            objects: objects,
            lights: lights,
            atmosphere: Atmosphere::earth((Vector::y()).normalize()),
            integrator: integrator,
        }
    }

    pub fn push(&mut self, o: Instance) {
        self.objects.push(o);
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        let mut result: Option<Intersection> = None;

        for s in &self.objects {
            result = s.intersect(ray).or(result)
        }

        result
    }
}
