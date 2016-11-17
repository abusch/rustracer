use std::sync::Arc;
use na::Norm;

use ::Vector;
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
               lights: Vec<Box<Light + Sync + Send>>)
               -> Scene {
        // let bvh = BVH::new(16, objects);
        Scene {
            camera: camera,
            // objects: objects,
            // bvh: bvh,
            lights: lights,
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
        }
    }

    // pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
    //     self.bvh.intersect(ray, |r, i| i.intersect(r))
    // }
    pub fn intersect2(&self, ray: &mut Ray) -> Option<SurfaceInteraction> {
        self.primitives.iter().fold(None, |r, p| p.intersect(ray).or(r))
    }

    pub fn intersect_p(&self, ray: &mut Ray) -> bool {
        self.intersect2(ray).is_some()
    }
}
