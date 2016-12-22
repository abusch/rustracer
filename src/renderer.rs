use std::io;
use std::path::Path;
use std::sync::mpsc::channel;

use crossbeam;
use img;

use Dim;
use block_queue::BlockQueue;
use filter::boxfilter::BoxFilter;
use film::Film;
use sampler::Sampler;
use sampler::zerotwosequence::ZeroTwoSequence;
use scene::Scene;
use spectrum::Spectrum;
use stats;

pub fn render(scene: Scene,
              dim: Dim,
              filename: &str,
              num_threads: usize,
              spp: usize,
              bs: u32)
              -> stats::Stats {
    let mut film = Film::new(dim, Box::new(BoxFilter {}));

    let block_size = bs;
    let block_queue = BlockQueue::new(dim, block_size);
    let (pixel_tx, pixel_rx) = channel();
    let (stats_tx, stats_rx) = channel();
    info!("Rendering scene using {} threads", num_threads);
    crossbeam::scope(|scope| {
        for _ in 0..num_threads {
            let scene = &scene;
            let pixel_tx = pixel_tx.clone();
            let stats_tx = stats_tx.clone();
            let bq = &block_queue;
            scope.spawn(move || {
                let mut sampler = ZeroTwoSequence::new(spp, 4);
                while let Some(block) = bq.next() {
                    info!("Rendering tile {}", block);
                    bq.report_progress();
                    for p in block {
                        sampler.start_pixel(&p);
                        loop {
                            let s = sampler.get_camera_sample();
                            let mut ray = scene.camera.ray_for(&s);
                            let sample_colour = scene.integrator
                                .li(scene, &mut ray, &mut sampler, 0);
                            let film_sample = FilmSample {
                                x: s.x,
                                y: s.y,
                                c: sample_colour,
                            };
                            pixel_tx.send(film_sample)
                                .expect(&format!("Failed to send sample {:?}", film_sample));
                            if !sampler.start_next_sample() {
                                break;
                            }
                        }
                    }
                }
                stats_tx.send(stats::get_stats()).expect("Failed to send thread stats");
            });
        }
    });

    // Write all pixels to the image
    for s in pixel_rx.iter()
        .take(block_queue.num_blocks as usize * block_size as usize * block_size as usize * spp) {
        film.add_sample(s.x, s.y, s.c);
    }
    // Collect all the stats from the threads
    let global_stats = stats_rx.iter().take(num_threads).fold(stats::get_stats(), |a, b| a + b);
    println!("");
    write_png(dim, &film.render(), filename)
        .expect(&format!("Could not write image to file {}", filename));

    global_stats
}

fn write_png(dim: Dim, image: &[Spectrum], filename: &str) -> io::Result<()> {
    let (w, h) = dim;
    let mut buffer = Vec::new();

    for i in 0..w * h {
        let bytes = image[i as usize].to_srgb();
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
    c: Spectrum,
}
