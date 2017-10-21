use {Point2f, Vector3f};
use ray::Ray;
use bounds::Bounds3f;
use interaction::{Interaction, SurfaceInteraction};

pub mod sphere;
pub mod disk;
pub mod mesh;
pub mod cylinder;

pub trait Shape {
    fn intersect(&self, ray: &Ray) -> Option<(SurfaceInteraction, f32)>;

    fn intersect_p(&self, ray: &Ray) -> bool {
        self.intersect(ray).is_some()
    }

    fn area(&self) -> f32;

    fn object_bounds(&self) -> Bounds3f;

    fn world_bounds(&self) -> Bounds3f;

    fn sample(&self, u: &Point2f) -> (Interaction, f32);

    fn sample_si(&self, si: &Interaction, u: &Point2f) -> (Interaction, f32) {
        let (intr, mut pdf) = self.sample(u);
        let mut wi = intr.p - si.p;
        let d2 = wi.length_squared();
        if d2 == 0.0 {
            pdf = 0.0;
        } else {
            wi = wi.normalize();
            pdf *= d2 / (intr.n.dot(&(-wi)).abs());
            if pdf.is_infinite() {
                pdf = 0.0;
            }
        }

        (intr, pdf)
    }

    fn pdf(&self, _si: &SurfaceInteraction) -> f32 {
        1.0 / self.area()
    }

    fn pdf_wi(&self, si: &SurfaceInteraction, wi: &Vector3f) -> f32 {
        let ray = si.spawn_ray(wi);

        if let Some((isect_light, _t_hit)) = self.intersect(&ray) {
            (si.p - isect_light.p).length_squared()
                / (isect_light.n.dot(&(-(*wi))).abs() * self.area())
        } else {
            0.0
        }
    }
}
