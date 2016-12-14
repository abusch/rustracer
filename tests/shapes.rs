extern crate rustracer as rt;
extern crate rand;
extern crate nalgebra as na;

use std::f32;
use rand::{Rng, StdRng, SeedableRng};
use rand::distributions::{Range, IndependentSample};
use na::{Norm, Dot};

use rt::{Point, Point2f};
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
        let sphere = Sphere::new().radius(radius);
        test_reintersection_convex(&sphere, &mut rng);
    }
}

fn test_reintersection_convex<T: Shape>(shape: &T, rng: &mut StdRng) {
    // Ray origin
    let o = Point::new(pexp(rng, 8.0), pexp(rng, 8.0), pexp(rng, 8.0));

    // Destination
    let bounds = shape.world_bounds();
    let t = Point::new(rng.next_f32(), rng.next_f32(), rng.next_f32());
    let p = bounds.lerp(&t);
    let mut ray = Ray::new(o, p - o);
    if rng.next_f32() < 0.5 {
        ray.d = ray.d.normalize();
    }

    // We usually, but not always, get an intersection
    if let Some((isect, _t_hit)) = shape.intersect(&mut ray) {
        // Now trace a bunch of rays leaving the intersection point
        for i in 0..10000 {
            // Random direction leaving the intersection point
            let u = Point2f::new(rng.next_f32(), rng.next_f32());
            let mut w = sampling::uniform_sample_sphere(&u);
            if w.dot(&isect.n) < 0.0 {
                w = -w;
            }
            let mut ray_out = isect.spawn_ray(&w);
            assert!(!shape.intersect_p(&mut ray_out));
            assert!(shape.intersect(&mut ray_out).is_none());

        }
    }
}
