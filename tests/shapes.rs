extern crate rand;
extern crate rustracer as rt;

use std::f32;
use rand::{Rng, SeedableRng, StdRng};
use rand::distributions::{IndependentSample, Range};

use rt::{Point2f, Point3f, Transform};
use rt::ray::Ray;
use rt::sampling;
use rt::shapes::Shape;
use rt::shapes::sphere::Sphere;

fn pexp<T: Rng>(rng: &mut T, exp: f32) -> f32 {
    let range = Range::new(-exp, exp);
    let logu: f32 = range.ind_sample(rng);
    let base = 10.0_f32;
    base.powf(logu)
}

#[test]
fn full_sphere_reintersect() {
    let mut rng = StdRng::from_seed(&[0]);
    for i in 0..1000 {
        rng.reseed(&[i]);
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
    let t = Point3f::new(rng.next_f32(), rng.next_f32(), rng.next_f32());
    let p = bounds.lerp(&t);
    let mut ray = Ray::new(o, p - o);
    if rng.next_f32() < 0.5 {
        ray.d = ray.d.normalize();
    }

    // We usually, but not always, get an intersection
    if let Some((isect, _t_hit)) = shape.intersect(&ray) {
        // Now trace a bunch of rays leaving the intersection point
        for _ in 0..1000 {
            // Random direction leaving the intersection point
            let u = Point2f::new(rng.next_f32(), rng.next_f32());
            let mut w = sampling::uniform_sample_sphere(&u);
            if w.dotn(&isect.n) < 0.0 {
                w = -w;
            }
            let ray_out = isect.spawn_ray(&w);
            assert!(!shape.intersect_p(&ray_out));
            assert!(shape.intersect(&ray_out).is_none());
        }
    }
}
