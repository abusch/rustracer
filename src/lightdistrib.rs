use ::Point3f;
use sampling::Distribution1D;
use scene::Scene;

pub trait LightDistribution {
    fn lookup<'a>(&'a self, p: &Point3f) -> &'a Distribution1D;
}

pub struct UniformLightDistribution {
    distrib: Box<Distribution1D>,
}

impl UniformLightDistribution {
    pub fn new(scene: &Scene) -> UniformLightDistribution {
        let prob = vec![1.0; scene.lights.len()];
        UniformLightDistribution { distrib: Box::new(Distribution1D::new(&prob[..])) }
    }
}

impl LightDistribution for UniformLightDistribution {
    fn lookup<'a>(&'a self, p: &Point3f) -> &'a Distribution1D {
        &self.distrib
    }
}
