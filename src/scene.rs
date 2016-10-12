use std::path::Path;

use camera::Camera;
use geometry::*;
use integrator::Integrator;
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
    pub camera: Camera,
    pub objects: Vec<Instance>,
    pub lights: Vec<Box<Light + Sync + Send>>,
    pub atmosphere: Atmosphere,
    pub integrator: Box<Integrator + Sync + Send>,
}

impl Scene {
    pub fn new(camera: Camera, integrator: Box<Integrator + Sync + Send>) -> Scene {
        Scene {
            camera: camera,
            objects: Vec::new(),
            lights: Vec::new(),
            atmosphere: Atmosphere::earth((Vector::y()).normalize()),
            integrator: integrator,
        }
    }

    pub fn push(&mut self, o: Instance) {
        self.objects.push(o);
    }

    pub fn push_sphere(&mut self, r: f32, sc: Colourf, tr: f32, rf: f32, transform: Transform) {
        self.push(Instance::new(Box::new(Sphere::new(r)),
                                Material::new(sc, tr, rf),
                                transform));
    }

    pub fn push_plane(&mut self, sc: Colourf, tr: f32, rf: f32, transform: Transform) {
        self.push(Instance::new(Box::new(Plane), Material::new(sc, tr, rf), transform));
    }

    pub fn push_triangle(&mut self, v0: Point, v1: Point, v2: Point) {
        self.push(Instance::new(Box::new(Triangle::new(v0, v1, v2)),
                                Material::new(Colourf::rgb(0.4, 0.5, 0.6), 0.0, 0.0),
                                one()))
    }

    pub fn push_mesh(&mut self, file: &Path, name: &str, transform: Transform) {
        let mesh = Mesh::load(file, name);
        self.push(Instance::new(Box::new(mesh),
                                Material::new(Colourf::rgb(0.4, 0.5, 0.6), 1.0, 1.0),
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
