use std::rc::Rc;

use sphere::Sphere;
use plane::Plane;
use intersection::Intersection;
use ray::Ray;
use instance::Instance;
use material::Material;
use Point;
use Vector;
use colour::Colourf;
use light::{Light, PointLight, DistantLight};

pub struct Scene {
    pub objects: Vec<Instance>,
    pub lights: Vec<Box<Light>>,
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

    pub fn push_plane(&mut self, point: Point, n: Vector, sc: Colourf, tr: f32, rf: f32) {
        self.push(Instance::new(Rc::new(Plane::new(point, n)), Material::new(sc, tr, rf)));
    }

    pub fn push_point_light(&mut self, pos: Point, ec: Colourf) {
        self.lights.push(Box::new(PointLight::new(pos, ec)));
    }

    pub fn push_distant_light(&mut self, dir: Vector, ec: Colourf) {
        self.lights.push(Box::new(DistantLight::new(dir, ec)));
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


