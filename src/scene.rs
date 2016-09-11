use std::rc::Rc;

use sphere::Sphere;
use plane::Plane;
use intersection::Intersection;
use ray::Ray;
use instance::Instance;
use material::Material;
use ::{Point, Transform, Vector};
use colour::Colourf;
use light::{Light, PointLight, DistantLight};
use skydome::Atmosphere;
use na::Norm;

pub struct Scene {
    pub objects: Vec<Instance>,
    pub lights: Vec<Box<Light>>,
    pub atmosphere: Atmosphere,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            objects: Vec::new(),
            lights: Vec::new(),
            atmosphere: Atmosphere::earth((-Vector::z()).normalize()),
        }
    }

    pub fn push(&mut self, o: Instance) {
        self.objects.push(o);
    }

    pub fn push_sphere(&mut self, r: f32, sc: Colourf, tr: f32, rf: f32, transform: Transform) {
        self.push(Instance::new(Rc::new(Sphere::new(r)),
                                Material::new(sc, tr, rf),
                                transform));
    }

    pub fn push_plane(&mut self, sc: Colourf, tr: f32, rf: f32, transform: Transform) {
        self.push(Instance::new(Rc::new(Plane), Material::new(sc, tr, rf), transform));
    }

    pub fn push_point_light(&mut self, pos: Point, ec: Colourf) {
        self.lights.push(Box::new(PointLight::new(pos, ec)));
    }

    pub fn push_distant_light(&mut self, dir: Vector, ec: Colourf) {
        self.lights.push(Box::new(DistantLight::new(dir, ec)));
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        let mut result: Option<Intersection> = None;

        for s in &self.objects {
            result = s.intersect(ray).or(result)
        }

        result
    }
}
