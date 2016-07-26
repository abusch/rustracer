extern crate raytracer;

use std::io;
use std::fs::File;
use std::io::prelude::*;

use raytracer::ray::Ray;
use raytracer::scene::Scene;
use raytracer::colour::Colourf;
use raytracer::camera::Camera;
use raytracer::image::Image;
use raytracer::point::Point;
use raytracer::*;

pub const MAX_RAY_DEPTH: u8 = 5;

fn trace(ray: &mut Ray, scene: &Scene, depth: u32) -> Colourf {

    if let Some(intersection) = scene.intersect(ray) {
        let hit = intersection.hit;
        let ref mat = hit.material;
        let mut surface_colour = Colourf::black();
        let phit = intersection.dg.phit;
        let mut nhit = intersection.dg.nhit;
        let bias = 1e-4;
        let mut inside = false;
        if ray.dir.dot(&nhit) > 0.0 {
            nhit = -nhit;
            inside = true;
        }

        if (mat.transparency > 0.0 || mat.reflection > 0.0) && depth < MAX_RAY_DEPTH as u32 {
            let facingratio = -ray.dir.dot(&nhit);
            let fresnel_effect = mix((1.0 - facingratio).powi(3), 1.0, 0.1);
            let refldir = (ray.dir - nhit * 2.0 * ray.dir.dot(&nhit)).normalize();
            let mut newray = Ray::new((phit + nhit*bias).as_point(), refldir);
            let reflection = trace(&mut newray, scene, depth + 1);
            let mut refraction = Colourf::black();
            if mat.transparency > 0.0 {
                let ior = 1.1;
                let eta = if inside { ior } else { 1.0 / ior };
                let cosi = -nhit.dot(&ray.dir);
                let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
                let refrdir = (ray.dir * eta + nhit * (eta * cosi - k.sqrt())).normalize();
                let mut newray = Ray::new((phit - nhit * bias).as_point(), refrdir);
                refraction = trace(&mut newray, scene, depth + 1);
            }

            surface_colour = mat.surface_colour * (reflection * fresnel_effect + refraction * (1.0 - fresnel_effect) * mat.transparency);
        } else {
            for l in &scene.lights {
                let mut transmission = Colourf::white();
                let light_direction = (l.pos - phit).normalize();
                // Check if this light is "visible" from the hit point
                for o in &scene.objects {
                    let mut new_ray = Ray::new((phit + nhit * bias).as_point(), light_direction);
                    if o.intersect(&mut new_ray).is_some() {
                        transmission = Colourf::black();
                        break;
                    }
                }
                surface_colour = surface_colour + mat.surface_colour * transmission * f32::max(0.0, nhit.dot(&light_direction)) * l.emission_colour;
            }
        }

        return surface_colour;
    } else {
        return Colourf::rgb(2.0, 2.0, 2.0);
    }
}

fn render(scene: &Scene) {
    let dim = (640, 480);
    let mut image = Image::new(dim);

    let camera = Camera::new(Point::origin(), dim, 30.0);

    for y in 0..dim.1 {
        for x in 0..dim.0 {
            let mut ray = camera.ray_for(x as f32, y as f32);
            image.write(x as u32, y as u32, trace(&mut ray, scene, 0));
        }
    }

    write_file(dim, image.buffer()).expect("Could not write file");
}

fn write_file(dim: Dim, image: &[Colourf]) -> Result<(), io::Error> {
    let (w, h) = dim;
    let mut f = try!(File::create("image.ppm"));

    try!(write!(&mut f, "P6\n{} {}\n255\n", w, h));
    for i in 0..w*h {
        let bytes: [u8; 3] = image[i as usize].into();
        try!(f.write(&bytes));
    }
    Ok(())
}

fn main() {
    let mut scene = Scene::new();

    scene.push_sphere(Point::new( 0.0, -10004.0, -20.0), 10000.0, Colourf::rgb(0.20, 0.20, 0.20), None,            0.0, 0.0);
    scene.push_sphere(Point::new( 0.0,      0.0, -20.0),     4.0, Colourf::rgb(1.00, 0.32, 0.36), None,            1.0, 0.5);
    scene.push_sphere(Point::new( 5.0,     -1.0, -15.0),     2.0, Colourf::rgb(0.90, 0.76, 0.46), None,            1.0, 0.0);
    scene.push_sphere(Point::new( 5.0,      0.0, -25.0),     3.0, Colourf::rgb(0.65, 0.77, 0.97), None,            1.0, 0.0);
    scene.push_sphere(Point::new(-5.5,      0.0, -15.0),     3.0, Colourf::rgb(0.90, 0.90, 0.90), None,            1.0, 0.0);
    //ight
    // scene.push_sphere(Point::new( 0.0,     20.0, -30.0),     3.0, Colourf::black(),               Some(Colourf::rgb(3.0, 3.0, 3.0)), 0.0, 0.0);
    scene.push_light(Point::new(0.0, 20.0, -30.0), Colourf::rgb(3.0, 3.0, 3.0));

    println!("Rendering scene...");
    let now = std::time::Instant::now();
    render(&scene);
    let duration = now.elapsed();
    println!("Scene rendered in {}s and {}ms)", duration.as_secs(), duration.subsec_nanos() / 1000000);
}
