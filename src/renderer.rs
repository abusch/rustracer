use std::io;
use std::sync::Arc;
use std::path::Path;
use std::sync::mpsc::channel;

use block_queue::BlockQueue;
use Dim;
use colour::Colourf;
use filter::mitchell::MitchellNetravali;
use image::Image;
use img;
use sampling::{Sampler, LowDiscrepancy};
use scene::Scene;
use tp::ThreadPool;

pub fn render(scene: Arc<Scene>,
              dim: Dim,
              filename: &str,
              num_threads: usize,
              spp: usize,
              bs: usize) {
    let mut image = Image::new(dim,
                               Box::new(MitchellNetravali::new(1.0, 1.0, 1.0 / 3.0, 1.0 / 3.0)));

    let block_size = bs;
    let block_queue = Arc::new(BlockQueue::new(dim, block_size));
    println!("Rendering scene using {} threads", num_threads);
    let pool = ThreadPool::new(num_threads);
    let (tx, rx) = channel();
    for _ in 0..num_threads {
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
    println!("");
    image.render();
    write_png(dim, image.buffer(), filename)
        .expect(&format!("Could not write image to file {}", filename));
}

fn write_png(dim: Dim, image: &[Colourf], filename: &str) -> io::Result<()> {
    let (w, h) = dim;
    let mut buffer = Vec::new();

    for i in 0..w * h {
        let bytes: [u8; 3] = image[i as usize].to_srgb().into();
        buffer.push(bytes[0]);
        buffer.push(bytes[1]);
        buffer.push(bytes[2]);
    }

    // Save the buffer
    img::save_buffer(&Path::new(filename),
                     &buffer,
                     w as u32,
                     h as u32,
                     img::RGB(8))
}

#[derive(Debug, Copy, Clone)]
struct ImageSample {
    x: f32,
    y: f32,
    c: Colourf,
}
