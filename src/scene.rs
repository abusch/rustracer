use std::rc::Rc;
use std::num;
use std::path::Path;

use geometry::*;
use intersection::Intersection;
use ray::Ray;
use instance::Instance;
use material::Material;
use ::{Point, Transform, Vector};
use colour::Colourf;
use light::{Light, PointLight, DistantLight};
use skydome::Atmosphere;
use na::{Norm, one};

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
            atmosphere: Atmosphere::earth((Vector::y()).normalize()),
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

    pub fn push_triangle(&mut self, v0: Point, v1: Point, v2: Point) {
        self.push(Instance::new(Rc::new(Triangle::new(v0, v1, v2)),
                                Material::new(Colourf::rgb(0.4, 0.5, 0.6), 0.0, 0.0),
                                one()))
    }

    pub fn push_mesh(&mut self, file: &Path, name: &str, transform: Transform) {
        let mesh = Mesh::load(file, name);
        self.push(Instance::new(Rc::new(mesh),
                                Material::new(Colourf::rgb(0.4, 0.5, 0.6), 0.0, 0.0),
                                transform));
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
