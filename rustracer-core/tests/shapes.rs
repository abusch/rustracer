use rand::{rngs::StdRng, Rng, SeedableRng};
use std::f32;

use rustracer_core::ray::Ray;
use rustracer_core::sampling;
use rustracer_core::shapes::{Shape, Sphere};
use rustracer_core::{Point2f, Point3f, Transform};

fn pexp<T: Rng>(rng: &mut T, exp: f32) -> f32 {
    // let range = Range::new(-exp, exp);
    let logu: f32 = rng.gen_range(-exp..=exp); // range.ind_sample(rng);
    let base = 10.0_f32;
    base.powf(logu)
}

#[test]
fn full_sphere_reintersect() {
    for i in 0..1000 {
        let mut rng = StdRng::seed_from_u64(i as u64);
        let radius = pexp(&mut rng, 4.0);
        let sphere = Sphere::new(Transform::default(), radius, -radius, radius, 360.0, false);
        test_reintersection_convex(&sphere, &mut rng);
    }
}

fn test_reintersection_convex<T: Shape>(shape: &T, rng: &mut StdRng) {
    // Ray origin
    let o = Point3f::new(pexp(rng, 8.0), pexp(rng, 8.0), pexp(rng, 8.0));

    // Destination
    let bounds = shape.world_bounds();
    let t = Point3f::new(rng.gen(), rng.gen(), rng.gen());
    let p = bounds.lerp(&t);
    let mut ray = Ray::new(o, p - o);
    if rng.gen::<f32>() < 0.5 {
        ray.d = ray.d.normalize();
    }

    // We usually, but not always, get an intersection
    if let Some((isect, _t_hit)) = shape.intersect(&ray) {
        // Now trace a bunch of rays leaving the intersection point
        for _ in 0..1000 {
            // Random direction leaving the intersection point
            let u = Point2f::new(rng.gen(), rng.gen());
            let mut w = sampling::uniform_sample_sphere(u);
            if w.dotn(&isect.hit.n) < 0.0 {
                w = -w;
            }
            let ray_out = isect.spawn_ray(&w);
            assert!(!shape.intersect_p(&ray_out));
            assert!(shape.intersect(&ray_out).is_none());
        }
    }
}
