use na::Dot;

use colour::Colourf;
use integrator::Integrator;
use ray::Ray;
use scene::Scene;

pub struct Normal {
}

impl Integrator for Normal {
    fn illumination(&self, scene: &Scene, ray: &mut Ray) -> Colourf {
        if let Some(intersection) = scene.intersect(ray) {
            let n = intersection.dg.nhit;
            Colourf::grey(ray.dir.dot(&n).abs())
        } else {
            Colourf::black()
        }

    }
}
