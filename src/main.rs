extern crate nalgebra as na;
extern crate rustracer as rt;
extern crate chrono;
extern crate docopt;

use std::f32::consts::*;
use std::f32;
use std::path::Path;
use std::sync::Arc;
use na::zero;
use chrono::*;
use docopt::Docopt;

use rt::scene::Scene;
use rt::colour::Colourf;
use rt::camera::Camera;
use rt::integrator::{Whitted, AmbientOcclusion};
use rt::{Point, Vector, Transform};
use rt::renderer;

const USAGE: &'static str = "
Toy Ray-Tracer in Rust

Usage:
  rustracer [options]

Options:
  -o <file>, --output=<file>  Output file name [default: image.png].
";

fn main() {
    // Parse args
    let args = Docopt::new(USAGE)
        .and_then(|d| d.parse())
        .unwrap_or_else(|e| e.exit());

    let filename = args.get_str("-o");

    // let dim = (1216, 1088);
    let dim = (800, 480);
    let camera = Camera::new(Point::new(0.0, 4.0, 0.0), dim, 50.0);
    // let integrator = Whitted::new(8);
    let integrator = AmbientOcclusion::new(64, f32::INFINITY);
    let mut scene = Scene::new(camera, Box::new(integrator));
    let height = 5.0;

    // scene.push_sphere(Point::new( 0.0, -10004.0, -20.0), 10000.0, Colourf::rgb(0.20, 0.20, 0.20), 0.0, 0.0);
    // scene.push_sphere(4.0,
    //                   Colourf::rgb(1.00, 0.32, 0.36),
    //                   0.8,
    //                   0.0,
    //                   Transform::new(Vector::new(0.0, height, -20.0), zero(), 1.0));
    // scene.push_sphere(2.0,
    //                   Colourf::rgb(0.90, 0.76, 0.46),
    //                   0.0,
    //                   0.0,
    //                   Transform::new(Vector::new(5.0, height - 1.0, -15.0), zero(), 1.0));
    scene.push_mesh(Path::new("models/smooth_suzanne.obj"),
                    "Suzanne",
                    Transform::new(Vector::new(2.0, height, -15.0),
                                   Vector::new(0.0, 0.0, 0.0),
                                   4.0));
    // scene.push_sphere(3.0,
    //                   Colourf::rgb(0.65, 0.77, 0.97),
    //                   0.0,
    //                   0.0,
    //                   Transform::new(Vector::new(5.0, height, -25.0), zero(), 1.0));
    scene.push_sphere(3.0,
                      Colourf::rgb(0.90, 0.90, 0.90),
                      0.0,
                      0.0,
                      Transform::new(Vector::new(-6.5, 4.0, -15.0), zero(), 1.0));
    // scene.push_triangle(Point::new(-1.0, height - 1.0, -5.0),
    //                     Point::new(1.0, height - 1.0, -5.0),
    //                     Point::new(0.0, height + 0.0, -8.0));
    scene.push_plane(Colourf::rgb(1.0, 1.0, 1.0),
                     0.0,
                     0.0,
                     Transform::new(Vector::new(0.0, height - 4.0, 0.0),
                                    Vector::new(-FRAC_PI_2, 0.0, 0.0),
                                    1.0));
    // Light
    // scene.push_sphere(Point::new( 0.0,     20.0, -30.0),     3.0, Colourf::black(),               Some(Colourf::rgb(3.0, 3.0, 3.0)), 0.0, 0.0);
    scene.push_point_light(Point::new(-10.0, 10.0, -5.0),
                           Colourf::rgb(3000.0, 2000.0, 2000.0));
    scene.push_distant_light(-Vector::y() - Vector::z(), Colourf::rgb(3.0, 3.0, 3.0));

    let duration = Duration::span(|| renderer::render(Arc::new(scene), dim, filename));
    let stats = rt::stats::get_stats();
    println!("Render time                : {}", duration);
    println!("Primary rays               : {}", stats.primary_rays);
    println!("Secondary rays             : {}", stats.secondary_rays);
    println!("Num triangles              : {}", stats.triangles);
    println!("Ray-triangle tests         : {}", stats.ray_triangle_tests);
    println!("Ray-triangle intersections : {}", stats.ray_triangle_isect);
    println!("Fast bounding-box test     : {}", stats.fast_bbox_isect);
}
