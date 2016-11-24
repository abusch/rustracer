use std::sync::Arc;
use std::mem;

use na::Norm;

use ::Vector;
use bounds::Bounds3f;
use camera::Camera;
use integrator::SamplerIntegrator;
use interaction::SurfaceInteraction;
use light::Light;
use primitive::Primitive;
use ray::Ray;
use skydome::Atmosphere;

pub struct Scene {
    pub camera: Camera,
    // bvh: BVH<Instance>,
    pub lights: Vec<Arc<Light + Sync + Send>>,
    pub atmosphere: Atmosphere,
    pub integrator: Box<SamplerIntegrator + Sync + Send>,
    pub primitives: Vec<Box<Primitive + Sync + Send>>,
}

impl Scene {
    pub fn new(camera: Camera,
               integrator: Box<SamplerIntegrator + Sync + Send>,
               primitives: Vec<Box<Primitive + Sync + Send>>,
               mut lights: Vec<Arc<Light + Sync + Send>>)
               -> Scene {
        // let bvh = BVH::new(16, objects);
        let mut scene = Scene {
            camera: camera,
            // objects: objects,
            // bvh: bvh,
            lights: Vec::new(),
            atmosphere: Atmosphere::earth((Vector::y()).normalize()),
            integrator: integrator,
            primitives: primitives,
        };
        // TODO There's a bit of a circular reference with AreaLight <-> Shape <-> GeometricPrimitive which
        // doesn't play well with mutation needed by preprocessing...
        // for l in lights.iter_mut() {
        //     l.borrow_mut().preprocess(&scene);
        // }
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
