extern crate nalgebra as na;
extern crate image;
extern crate raytracer;

use std::io;
use std::path::Path;
use na::Origin;

use raytracer::scene::Scene;
use raytracer::colour::Colourf;
use raytracer::camera::Camera;
use raytracer::image::Image;
use raytracer::integrator::{Integrator, Whitted};
use raytracer::{Dim, Point, Vector};

pub const MAX_RAY_DEPTH: u8 = 8;

fn render(scene: &Scene) {
    let dim = (640, 480);
    let mut image = Image::new(dim);

    let integrator = Whitted::new(8);
    let camera = Camera::new(Point::origin(), dim, 30.0);
    // let samples = [(0.25, 0.25), (0.25, 0.75), (0.75, 0.75), (0.75, 0.25)];
    let samples = [(0.5, 0.5)];
    let spp = 1.0;

    for y in 0..dim.1 {
        for x in 0..dim.0 {
            let mut c = Colourf::black();
            for s in &samples {
                let mut ray = camera.ray_for(x as f32 + s.0, y as f32 + s.1);
                // c += trace(&mut ray, scene, 0);
                c += integrator.illumination(scene, &mut ray);
            }
            image.write(x as u32, y as u32, c / spp);
        }
    }

    write_png(dim, image.buffer()).expect("Could not write file");
}

fn write_png(dim: Dim, image: &[Colourf]) -> io::Result<()> {
    let (w, h) = dim;
    let mut buffer = Vec::new();

    for i in 0..w*h {
        let bytes: [u8; 3] = image[i as usize].into();
        buffer.push(bytes[0]);
        buffer.push(bytes[1]);
        buffer.push(bytes[2]);
    }

    // Save the buffer as "image.png"
    image::save_buffer(&Path::new("image.png"), &buffer, w, h, image::RGB(8))
}

fn main() {
    let mut scene = Scene::new();

    // scene.push_sphere(Point::new( 0.0, -10004.0, -20.0), 10000.0, Colourf::rgb(0.20, 0.20, 0.20), 0.0, 0.0);
    scene.push_sphere(Point::new( 0.0,      0.0, -20.0),     4.0, Colourf::rgb(1.00, 0.32, 0.36), 1.0, 0.5);
    scene.push_sphere(Point::new( 5.0,     -1.0, -15.0),     2.0, Colourf::rgb(0.90, 0.76, 0.46), 1.0, 0.0);
    scene.push_sphere(Point::new( 5.0,      0.0, -25.0),     3.0, Colourf::rgb(0.65, 0.77, 0.97), 1.0, 0.0);
    scene.push_sphere(Point::new(-5.5,      0.0, -15.0),     3.0, Colourf::rgb(0.90, 0.90, 0.90), 1.0, 0.0);
    scene.push_plane(Point::new(0.0, -50.0, 0.0), Vector::new(0.0, 1.0, 0.0), Colourf::rgb(1.0, 1.0, 1.0), 0.0, 0.0);
    //ight
    // scene.push_sphere(Point::new( 0.0,     20.0, -30.0),     3.0, Colourf::black(),               Some(Colourf::rgb(3.0, 3.0, 3.0)), 0.0, 0.0);
    scene.push_point_light(Point::new(-10.0, 10.0, -5.0), Colourf::rgb(3000.0, 0.0, 3000.0));
    scene.push_distant_light(Vector::new(-1.0, -1.0, -1.0), Colourf::rgb(3.0, 3.0, 3.0));

    println!("Rendering scene...");
    let now = std::time::Instant::now();
    render(&scene);
    let duration = now.elapsed();
    println!("Scene rendered in {}s and {}ms", duration.as_secs(), duration.subsec_nanos() / 1000000);
}
