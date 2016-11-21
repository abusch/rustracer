use na::{Dot, Norm};

use {Point2f, Vector};
use ray::Ray;
use bounds::Bounds3f;
use interaction::{Interaction, SurfaceInteraction};

pub mod sphere;
pub mod disk;

pub trait Shape {
    fn intersect(&self, ray: &Ray) -> Option<(SurfaceInteraction, f32)>;

    fn intersect_p(&self, ray: &Ray) -> bool {
        self.intersect(ray).is_some()
    }

    fn area(&self) -> f32 {
        0.0
    }

    fn object_bounds(&self) -> Bounds3f;

    fn world_bounds(&self) -> Bounds3f;

    fn sample(&self, u: &Point2f) -> Interaction;

    fn sample_si(&self, si: &SurfaceInteraction, u: &Point2f) -> Interaction {
        self.sample(u)
    }

    fn pdf(&self, si: &SurfaceInteraction) -> f32 {
        1.0 / self.area()
    }

    fn pdf_wi(&self, si: &SurfaceInteraction, wi: &Vector) -> f32 {
        let ray = si.spawn_ray(wi);

        if let Some((isect_light, t_hit)) = self.intersect(&ray) {
            (si.p - isect_light.p).norm_squared() /
            (isect_light.n.dot(&(-(*wi))).abs() * self.area())
        } else {
            0.0
        }
    }
}
