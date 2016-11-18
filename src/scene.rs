use std::mem;
use std::sync::Arc;
use na::Norm;

use ::Vector;
use bounds::Bounds3f;
use bvh::BVH;
use camera::Camera;
use colour::Colourf;
use instance::Instance;
use integrator::SamplerIntegrator;
use intersection::Intersection;
use interaction::SurfaceInteraction;
use light::Light;
use material::MatteMaterial;
use ray::Ray;
use skydome::Atmosphere;
use primitive::{Primitive, GeometricPrimitive};
use shapes::sphere::Sphere;

pub struct Scene {
    pub camera: Camera,
    // bvh: BVH<Instance>,
    pub lights: Vec<Box<Light + Sync + Send>>,
    pub atmosphere: Atmosphere,
    pub integrator: Box<SamplerIntegrator + Sync + Send>,
    pub primitives: Vec<Box<Primitive + Sync + Send>>,
}

impl Scene {
    pub fn new(camera: Camera,
               integrator: Box<SamplerIntegrator + Sync + Send>,
               objects: &mut Vec<Instance>,
               mut lights: Vec<Box<Light + Sync + Send>>)
               -> Scene {
        // let bvh = BVH::new(16, objects);
        let mut scene = Scene {
            camera: camera,
            // objects: objects,
            // bvh: bvh,
            lights: Vec::new(),
            atmosphere: Atmosphere::earth((Vector::y()).normalize()),
            integrator: integrator,
            primitives: vec![Box::new(GeometricPrimitive {
                                 shape: Arc::new(Sphere::default()),
                                 area_light: None,
                                 material: Some(Arc::new(MatteMaterial::new(Colourf::rgb(1.0,
                                                                                         0.0,
                                                                                         0.0),
                                                                            20.0))),
                             })],
        };
        for l in lights.iter_mut() {
            l.preprocess(&scene);
        }
        mem::replace(&mut scene.lights, lights);

        scene
    }

    // pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
    //     self.bvh.intersect(ray, |r, i| i.intersect(r))
    // }
    pub fn intersect2(&self, ray: &mut Ray) -> Option<SurfaceInteraction> {
        self.primitives.iter().fold(None, |r, p| p.intersect(ray).or(r))
    }

    pub fn intersect_p(&self, ray: &Ray) -> bool {
        // self.intersect2(ray).is_some()
        self.primitives.iter().fold(false, |r, p| p.intersect_p(ray) || r)
    }

    pub fn world_bounds(&self) -> Bounds3f {
        self.primitives.iter().fold(Bounds3f::new(),
                                    |r, p| Bounds3f::union(&r, &p.world_bounds()))
    }
}
