use std::io;
use std::path::Path;
use std::sync::mpsc::channel;

use crossbeam;
use img;
use minifb;

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
              block_size: u32)
              -> stats::Stats {
    let film = Film::new(dim, Box::new(BoxFilter {}));

    let block_queue = BlockQueue::new(dim, block_size);
    let num_blocks = block_queue.num_blocks;
    // This channel will receive tiles of sampled pixels
    let (pixel_tx, pixel_rx) = channel();
    // This channel will receive the stats from each worker thread
    let (stats_tx, stats_rx) = channel();
    info!("Rendering scene using {} threads", num_threads);
    crossbeam::scope(|scope| {
        let film = &film;
        let scene = &scene;
        let bq = &block_queue;

        // Spawn thread to collect pixels and render image to file
        scope.spawn(move || {
            let mut window = minifb::Window::new("Rustracer",
                                                 dim.0 as usize,
                                                 dim.1 as usize,
                                                 minifb::WindowOptions::default())
                .expect("Unable to open a window");
            // Write all tiles to the image
            info!("Receiving tiles...");
            for _ in 0..num_blocks {
                let tile = pixel_rx.recv().unwrap();
                &film.merge_film_tile(tile);
                update_display(&mut window, &film);
            }
        });

        // Spawn worker threads
        for _ in 0..num_threads {
            let pixel_tx = pixel_tx.clone();
            let stats_tx = stats_tx.clone();
            scope.spawn(move || {
                let mut sampler = ZeroTwoSequence::new(spp, 4);
                while let Some(block) = bq.next() {
                    info!("Rendering tile {}", block);
                    let seed = block.start.y / bq.block_size * bq.dims.0 +
                               block.start.x / bq.block_size;
                    sampler.reseed(seed as u64);
                    let mut tile = film.get_film_tile(&block.bounds());
                    bq.report_progress();
                    for p in &tile.get_pixel_bounds() {
                        sampler.start_pixel(&p);
                        loop {
                            let s = sampler.get_camera_sample(&p);
                            let mut ray = scene.camera.generate_ray(&s);
                            let sample_colour = scene.integrator
                                .li(scene, &mut ray, &mut sampler, 0);
                            tile.add_sample(&s.p_film, sample_colour);
                            if !sampler.start_next_sample() {
                                break;
                            }
                        }
                    }
                    // Once we've rendered all the samples for the tile, send the tile through the
                    // channel to the main thread which will add it to the film.
                    pixel_tx.send(tile)
                        .expect(&format!("Failed to send tile"));
                }
                // Once there are no more tiles to render, send the thread's accumulated stats back
                // to the main thread
                stats_tx.send(stats::get_stats()).expect("Failed to send thread stats");
            });
        }
    });

    // Collect all the stats from the threads
    let global_stats = stats_rx.iter().take(num_threads).fold(stats::get_stats(), |a, b| a + b);
    println!("");

    write_png(dim, &film.render(), filename)
        .expect(&format!("Could not write image to file {}", filename));

    global_stats
}

fn update_display(window: &mut minifb::Window, film: &Film) {
    let buffer: Vec<u32> = film.render()
        .iter()
        .map(|p| {
            let rgb = p.to_srgb();
            (rgb[0] as u32) << 16 | (rgb[1] as u32) << 8 | (rgb[2] as u32)

        })
        .collect();

    window.update_with_buffer(&buffer[..]);
}

fn write_png(dim: Dim, image: &[Spectrum], filename: &str) -> io::Result<()> {
    let (w, h) = dim;
    let mut buffer = Vec::new();

    info!("Converting image to sRGB");
    for i in 0..w * h {
        let bytes = image[i as usize].to_srgb();
        buffer.push(bytes[0]);
        buffer.push(bytes[1]);
        buffer.push(bytes[2]);
    }

    // Save the buffer
    info!("Writing image to file {}", filename);
    img::save_buffer(&Path::new(filename),
                     &buffer,
                     w as u32,
                     h as u32,
                     img::RGB(8))
}
