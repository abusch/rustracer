extern crate nalgebra as na;
extern crate image;
extern crate raytracer as rt;
extern crate threadpool as tp;
extern crate chrono;

use std::io;
use std::io::Write;
use std::f32::consts::*;
use std::f32;
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use na::{zero, Point2};
use tp::ThreadPool;
use chrono::*;

use rt::scene::Scene;
use rt::colour::Colourf;
use rt::camera::Camera;
use rt::filter::mitchell::MitchellNetravali;
use rt::image::Image;
use rt::integrator::{Whitted, AmbientOcclusion};
use rt::{Dim, Point, Vector, Transform};
use rt::sampling::{Sampler, LowDiscrepancy};

#[derive(Debug, Copy, Clone)]
struct ImageSample {
    x: f32,
    y: f32,
    c: Colourf,
}

#[derive(Debug)]
struct Block {
    start: na::Point2<usize>,
    current: na::Point2<usize>,
    end: na::Point2<usize>,
}

impl Block {
    fn new(start: (usize, usize), size: usize) -> Block {
        Block {
            start: na::Point2::new(start.0, start.1),
            current: na::Point2::new(start.0, start.1),
            end: na::Point2::new(start.0 + size - 1, start.1 + size - 1),
        }
    }
}

impl Iterator for Block {
    type Item = na::Point2<usize>;

    fn next(&mut self) -> Option<Point2<usize>> {
        if self.current.x > self.end.x || self.current.y > self.end.y {
            None
        } else {

            let cur = self.current;

            if self.current.x == self.end.x {
                self.current.x = self.start.x;
                self.current.y += 1;
            } else {
                self.current.x += 1;
            }

            Some(cur)
        }
    }
}

struct BlockQueue {
    dims: (usize, usize),
    block_size: usize,
    counter: AtomicUsize,
    num_blocks: usize,
}

impl BlockQueue {
    pub fn new(dims: (usize, usize), block_size: usize) -> BlockQueue {
        BlockQueue {
            dims: dims,
            block_size: block_size,
            counter: ATOMIC_USIZE_INIT,
            num_blocks: (dims.0 / block_size) * (dims.1 / block_size),
        }
    }

    fn next(&self) -> Option<Block> {
        let c = self.counter.fetch_add(1, Ordering::AcqRel);
        if c >= self.num_blocks {
            None
        } else {
            let num_blocks_width = self.dims.0 / self.block_size;
            Some(Block::new((c % num_blocks_width * self.block_size,
                             c / num_blocks_width * self.block_size),
                            self.block_size))
        }
    }

    pub fn report_progress(&self) {
        print!("\rRendering block {}/{}...  ",
               self.counter.load(Ordering::Relaxed),
               self.num_blocks);
        io::stdout().flush().expect("Could not flush stdout");;
    }
}

fn render(scene: Arc<Scene>, dim: Dim) {
    let mut image = Image::new(dim,
                               Box::new(MitchellNetravali::new(1.0, 1.0, 1.0 / 3.0, 1.0 / 3.0)));

    let spp = 4;
    let num_workers = 8;
    let block_size = 32;
    let block_queue = Arc::new(BlockQueue::new(dim, block_size));
    println!("Using {} threads", num_workers);
    let pool = ThreadPool::new(num_workers);
    let (tx, rx) = channel();
    for _ in 0..num_workers {
        let scene = scene.clone();
        let tx = tx.clone();
        let block_queue = block_queue.clone();
        pool.execute(move || {
            let mut samples = Vec::new();
            samples.resize(spp, (0.0, 0.0));
            let sampler = LowDiscrepancy::new(spp);
            while let Some(block) = block_queue.next() {
                block_queue.report_progress();
                for p in block {
                    sampler.get_samples(p.x as f32, p.y as f32, &mut samples);
                    for s in &samples {
                        let mut ray = scene.camera.ray_for(s.0, s.1);
                        let sample_colour = scene.integrator.illumination(&scene, &mut ray);
                        let image_sample = ImageSample {
                            x: s.0,
                            y: s.1,
                            c: sample_colour,
                        };
                        tx.send(image_sample)
                            .expect(&format!("Failed to send sample {:?}", image_sample));
                    }
                }
            }
        });
    }

    for s in rx.iter().take(block_queue.num_blocks * block_size * block_size * spp) {
        image.add_sample(s.x, s.y, s.c);
    }
    print!("\n");
    image.render();
    write_png(dim, image.buffer()).expect("Could not write file");
}

fn write_png(dim: Dim, image: &[Colourf]) -> io::Result<()> {
    let (w, h) = dim;
    let mut buffer = Vec::new();

    for i in 0..w * h {
        let bytes: [u8; 3] = image[i as usize].to_srgb().into();
        buffer.push(bytes[0]);
        buffer.push(bytes[1]);
        buffer.push(bytes[2]);
    }

    // Save the buffer as "image.png"
    image::save_buffer(&Path::new("image.png"),
                       &buffer,
                       w as u32,
                       h as u32,
                       image::RGB(8))
}

fn main() {
    let dim = (800, 480);
    let camera = Camera::new(Point::new(0.0, 4.0, 0.0), dim, 50.0);
    // let integrator = Whitted::new(8);
    let integrator = AmbientOcclusion::new(128, f32::INFINITY);
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

    println!("Rendering scene...");
    let duration = Duration::span(|| render(Arc::new(scene), dim));
    let stats = rt::stats::get_stats();
    println!("Render time                : {}", duration);
    println!("Primary rays               : {}", stats.primary_rays);
    println!("Secondary rays             : {}", stats.secondary_rays);
    println!("Num triangles              : {}", stats.triangles);
    println!("Ray-triangle tests         : {}", stats.ray_triangle_tests);
    println!("Ray-triangle intersections : {}", stats.ray_triangle_isect);
    println!("Fast bounding-box test     : {}", stats.fast_bbox_isect);
}
