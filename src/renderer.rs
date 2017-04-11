extern crate pbr;

use std::path::Path;
use std::sync::mpsc::channel;

use crossbeam;
use img;

use Dim;
use block_queue::BlockQueue;
use display::DisplayUpdater;
use errors::*;
use integrator::SamplerIntegrator;
use sampler::Sampler;
use sampler::zerotwosequence::ZeroTwoSequence;
use scene::Scene;
use spectrum::Spectrum;
use stats;

pub fn render(scene: Scene,
              integrator: Box<SamplerIntegrator + Send + Sync>,
              dim: Dim,
              filename: &str,
              num_threads: usize,
              spp: usize,
              block_size: u32,
              mut display: Box<DisplayUpdater + Send>)
              -> Result<stats::Stats> {
    let block_queue = BlockQueue::new(dim, block_size);
    let num_blocks = block_queue.num_blocks;
    // This channel will receive tiles of sampled pixels
    let (pixel_tx, pixel_rx) = channel();
    // This channel will receive the stats from each worker thread
    let (stats_tx, stats_rx) = channel();
    info!("Rendering scene using {} threads", num_threads);
    crossbeam::scope(|scope| {
        let scene = &scene;
        let bq = &block_queue;
        let integrator = &integrator;

        // Spawn thread to collect pixels and render image to file
        scope.spawn(move || {
            // Write all tiles to the image
            let mut pb = pbr::ProgressBar::new(num_blocks as _);
            info!("Receiving tiles...");
            for _ in 0..num_blocks {
                let tile = pixel_rx.recv().unwrap();
                scene.camera.get_film().merge_film_tile(tile);
                pb.inc();
                display.update(scene.camera.get_film());
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
                    let seed = block.start.y as u32 / bq.block_size * bq.dims.0 +
                               block.start.x as u32 / bq.block_size;
                    sampler.reseed(seed as u64);
                    let mut tile = scene.camera.get_film().get_film_tile(&block.bounds());
                    for p in &tile.get_pixel_bounds() {
                        sampler.start_pixel(&p);
                        loop {
                            let s = sampler.get_camera_sample(&p);
                            let mut ray = scene.camera.generate_ray_differential(&s);
                            ray.scale_differentials(1.0 / (sampler.spp() as f32).sqrt());
                            let sample_colour = integrator.li(scene, &mut ray, &mut sampler, 0);
                            tile.add_sample(&s.p_film, sample_colour);
                            if !sampler.start_next_sample() {
                                break;
                            }
                        }
                    }
                    // Once we've rendered all the samples for the tile, send the tile through the
                    // channel to the main thread which will add it to the film.
                    pixel_tx
                        .send(tile)
                        .unwrap_or_else(|e| error!("Failed to send tile: {}", e));
                }
                // Once there are no more tiles to render, send the thread's accumulated stats back
                // to the main thread
                stats_tx
                    .send(stats::get_stats())
                    .unwrap_or_else(|e| error!("Failed to send thread stats: {}", e));
            });
        }
    });

    // Collect all the stats from the threads
    let global_stats = stats_rx
        .iter()
        .take(num_threads)
        .fold(stats::get_stats(), |a, b| a + b);

    write_png(dim, scene.camera.get_film().render().as_slice(), filename).map(|_| global_stats)
}

fn write_png(dim: Dim, image: &[Spectrum], filename: &str) -> Result<()> {
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
            .chain_err(|| format!("Failed to save image file {}", filename))
}
