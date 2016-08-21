use std::rc::Rc;

use sphere::Sphere;
use intersection::Intersection;
use ray::Ray;
use instance::Instance;
use material::Material;
use Point;
use colour::Colourf;
use light::Light;

pub struct Scene {
    pub objects: Vec<Instance>,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            objects: Vec::new(),
            lights: Vec::new(),
        }
    }

    pub fn push(&mut self, o: Instance) {
        self.objects.push(o);
    }

    pub fn push_sphere(&mut self, point: Point, r: f32, sc: Colourf, tr: f32, rf: f32) {
        self.push(Instance::new(Rc::new(Sphere::new(point, r)), Material::new(sc, tr, rf)));
    }

    pub fn push_light(&mut self, pos: Point, ec: Colourf) {
        self.lights.push(Light::new(pos, ec));
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        let mut result: Option<Intersection> = None;

        for i in 0..self.objects.len() {
            let s = &self.objects[i];
            result = s.intersect(ray).or(result)
        }

        result
    }
}


