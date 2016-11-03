use std::io;
use std::sync::Arc;
use std::path::Path;
use std::sync::mpsc::channel;

use tp::ThreadPool;
use img;

use Dim;
use block_queue::BlockQueue;
use colour::Colourf;
use filter::mitchell::MitchellNetravali;
use film::Film;
use sampling::{Sampler, LowDiscrepancy};
use scene::Scene;
use stats;

pub fn render(scene: Arc<Scene>,
              dim: Dim,
              filename: &str,
              num_threads: usize,
              spp: usize,
              bs: usize)
              -> stats::Stats {
    let mut film = Film::new(dim,
                             Box::new(MitchellNetravali::new(2.0, 2.0, 1.0 / 3.0, 1.0 / 3.0)));

    let block_size = bs;
    let block_queue = Arc::new(BlockQueue::new(dim, block_size));
    println!("Rendering scene using {} threads", num_threads);
    let pool = ThreadPool::new(num_threads);
    let (pixel_tx, pixel_rx) = channel();
    let (stats_tx, stats_rx) = channel();
    for _ in 0..num_threads {
        let scene = scene.clone();
        let pixel_tx = pixel_tx.clone();
        let stats_tx = stats_tx.clone();
        let block_queue = block_queue.clone();
        pool.execute(move || {
            let mut samples = Vec::new();
            samples.resize(spp, (0.0, 0.0));
            let mut sampler = LowDiscrepancy::new(spp);
            while let Some(block) = block_queue.next() {
                block_queue.report_progress();
                for p in block {
                    sampler.get_samples(p.x as f32, p.y as f32, &mut samples);
                    for s in &samples {
                        let mut ray = scene.camera.ray_for(s.0, s.1);
                        let sample_colour = scene.integrator.li(&scene, &mut ray, &mut sampler, 0);
                        let film_sample = FilmSample {
                            x: s.0,
                            y: s.1,
                            c: sample_colour,
                        };
                        pixel_tx.send(film_sample)
                            .expect(&format!("Failed to send sample {:?}", film_sample));
                    }
                }
            }
            stats_tx.send(stats::get_stats()).expect("Failed to send thread stats");
        });
    }

    // Write all pixels to the image
    for s in pixel_rx.iter().take(block_queue.num_blocks * block_size * block_size * spp) {
        film.add_sample(s.x, s.y, s.c);
    }
    // Collect all the stats from the threads
    let global_stats = stats_rx.iter().take(num_threads).fold(stats::get_stats(), |a, b| a + b);
    println!("");
    write_png(dim, &film.render(), filename)
        .expect(&format!("Could not write image to file {}", filename));

    global_stats
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
struct FilmSample {
    x: f32,
    y: f32,
    c: Colourf,
}
