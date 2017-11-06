use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

use crossbeam;
use indicatif;

use Point2i;
use bounds::Bounds2i;
use camera::Camera;
use display::DisplayUpdater;
use errors::*;
use integrator::SamplerIntegrator;
use light_arena::MemoryArena;
use sampler::Sampler;
use scene::Scene;
use stats;

pub fn render(scene: Box<Scene>,
              integrator: &mut Box<SamplerIntegrator + Send + Sync>,
              camera: Box<Camera + Send + Sync>,
              num_threads: usize,
              sampler: &mut Box<Sampler + Send + Sync>,
              block_size: i32,
              mut _display: Box<DisplayUpdater + Send>)
              -> Result<stats::Stats> {
    integrator.preprocess(&scene, sampler);
    let sample_bounds = camera.get_film().get_sample_bounds();
    let sample_extent = sample_bounds.diagonal();
    let pixel_bounds = &sample_bounds; // FIXME
    info!("Rendering with sample_bounds = {}, pixel_bounds = {}",
          sample_bounds,
          pixel_bounds);
    let n_tiles = Point2i::new((sample_extent.x + block_size - 1) / block_size,
                               (sample_extent.y + block_size - 1) / block_size);
    info!("n_tiles = {}", n_tiles);

    let num_blocks = n_tiles.x * n_tiles.y;
    // This channel will receive the stats from each worker thread
    let (stats_tx, stats_rx) = channel();
    info!("Rendering scene using {} threads", num_threads);
    let image_bounds = Bounds2i::from_points(&Point2i::new(0, 0),
                                             &Point2i::new(n_tiles.x, n_tiles.y));
    let tiles_iter = Arc::new(Mutex::new(image_bounds.into_iter()));
    let pb = indicatif::ProgressBar::new(num_blocks as _);
    pb.set_style(indicatif::ProgressStyle::default_bar()
        .progress_chars("=>-")
        .template("[{elapsed_precise}] [{wide_bar}] {percent}% [{pos}/{len}] {eta}")
    );

    crossbeam::scope(|scope| {
        // We only want to use references to these in the thread, not move the structs themselves...
        let scene = &scene;
        let integrator = &integrator;
        let camera = &camera;
        let pb = &pb;

        // Spawn worker threads
        for _ in 0..num_threads {
            let stats_tx = stats_tx.clone();
            let mut sampler = sampler.clone();
            let tiles_iter = Arc::clone(&tiles_iter);
            scope.spawn(move || {
                loop {
                    let maybe_tile = {
                        let mut iter = tiles_iter.lock().unwrap();
                        iter.next()
                    };
                    let tile = if let Some(t) = maybe_tile {
                        t
                    } else {
                        break;
                    };
                    // Render section of image corresponding to `tile`
                    info!("Tile number: {}", tile);

                    // Allocate MemoryArena for tile
                    let mut arena = MemoryArena::new(1);

                    // Get sampler instance for tile
                    let seed = tile.y * n_tiles.x + tile.x;
                    sampler.reseed(seed as u64);

                    // Compute sample bounds for tile
                    let x0 = sample_bounds.p_min.x + tile.x * block_size;
                    let x1 = i32::min(x0 + block_size, sample_bounds.p_max.x);
                    let y0 = sample_bounds.p_min.y + tile.y * block_size;
                    let y1 = i32::min(y0 + block_size, sample_bounds.p_max.y);
                    let tile_bounds = Bounds2i::from_points(&Point2i::new(x0, y0),
                                                            &Point2i::new(x1, y1));
                    info!("Starting image tile {}", tile_bounds);

                    let mut film_tile = camera.get_film().get_film_tile(&tile_bounds);
                    for p in &tile_bounds {
                        sampler.start_pixel(&p);

                        // Do this check after the start_pixel() call; this keeps
                        // the usage of RNG values from (most) Samplers that use
                        // RNGs consistent, which improves reproducability /
                        // debugging
                        if !pixel_bounds.inside_exclusive(&p) {
                            continue;
                        }

                        loop {
                            let alloc = arena.allocator();
                            let s = sampler.get_camera_sample(&p);
                            let mut ray = camera.generate_ray_differential(&s);
                            ray.scale_differentials(1.0 / (sampler.spp() as f32).sqrt());
                            stats::inc_camera_ray();
                            let sample_colour =
                                integrator.li(scene, &mut ray, &mut sampler, &alloc, 0);
                            film_tile.add_sample(&s.p_film, sample_colour);
                            if !sampler.start_next_sample() {
                                break;
                            }
                        }
                    }
                    camera.get_film().merge_film_tile(film_tile);
                    pb.inc(1);
                }
                // Once there are no more tiles to render, send the thread's accumulated stats back
                // to the main thread
                stats_tx
                    .send(stats::get_stats())
                    .unwrap_or_else(|e| error!("Failed to send thread stats: {}", e));
            });
        }
    });
    pb.finish();

    // Collect all the stats from the threads
    let global_stats = stats_rx
        .iter()
        .take(num_threads)
        .fold(stats::get_stats(), |a, b| a + b);

    camera.get_film().write_image().map(|_| global_stats)
}
