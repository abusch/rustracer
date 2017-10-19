extern crate pbr;

use std::sync::mpsc::channel;

use crossbeam;

use block_queue::BlockQueue;
use camera::Camera;
use display::DisplayUpdater;
use errors::*;
use integrator::SamplerIntegrator;
use light_arena::MemoryArena;
use sampler::Sampler;
use scene::Scene;
use stats;

pub fn render(
    scene: Box<Scene>,
    integrator: &mut Box<SamplerIntegrator + Send + Sync>,
    camera: Box<Camera + Send + Sync>,
    num_threads: usize,
    sampler: &mut Box<Sampler + Send + Sync>,
    block_size: i32,
    mut display: Box<DisplayUpdater + Send>,
) -> Result<stats::Stats> {
    integrator.preprocess(&scene, sampler);
    let res = camera.get_film().full_resolution;
    info!("Rendering with resolution {}", res);
    let block_queue = BlockQueue::new(res, block_size);
    let num_blocks = block_queue.num_blocks;
    // This channel will receive tiles of sampled pixels
    let (pixel_tx, pixel_rx) = channel();
    // This channel will receive the stats from each worker thread
    let (stats_tx, stats_rx) = channel();
    info!("Rendering scene using {} threads", num_threads);
    crossbeam::scope(|scope| {
        // We only want to use references to these in the thread, not move the structs themselves...
        let scene = &scene;
        let bq = &block_queue;
        let integrator = &integrator;
        let camera = &camera;

        // Spawn thread to collect pixels and render image to file
        scope.spawn(move || {
            // Write all tiles to the image
            let mut pb = pbr::ProgressBar::new(num_blocks as _);
            info!("Receiving tiles...");
            for _ in 0..num_blocks {
                let tile = pixel_rx.recv().unwrap();
                camera.get_film().merge_film_tile(tile);
                pb.inc();
                display.update(camera.get_film());
            }
        });

        // Spawn worker threads
        for _ in 0..num_threads {
            let pixel_tx = pixel_tx.clone();
            let stats_tx = stats_tx.clone();
            let mut sampler = sampler.clone();
            scope.spawn(move || {
                // let mut sampler = ZeroTwoSequence::new(spp, 4);
                let mut arena = MemoryArena::new(1);
                while let Some(block) = bq.next() {
                    info!("Rendering tile {}", block);
                    let seed =
                        block.start.y / bq.block_size * bq.dims.x + block.start.x / bq.block_size;
                    sampler.reseed(seed as u64);
                    let mut tile = camera.get_film().get_film_tile(&block.bounds());
                    for p in &tile.get_pixel_bounds() {
                        sampler.start_pixel(&p);
                        loop {
                            let alloc = arena.allocator();
                            let s = sampler.get_camera_sample(&p);
                            let mut ray = camera.generate_ray_differential(&s);
                            ray.scale_differentials(1.0 / (sampler.spp() as f32).sqrt());
                            let sample_colour =
                                integrator.li(scene, &mut ray, &mut sampler, 0, &alloc);
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

    camera.get_film().write_png().map(|_| global_stats)
}
