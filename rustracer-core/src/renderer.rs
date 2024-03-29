use std::sync::Arc;

use anyhow::Result;
use light_arena::MemoryArena;
use log::{error, info};
use parking_lot::Mutex;

use crate::bounds::Bounds2i;
use crate::camera::Camera;
use crate::integrator::SamplerIntegrator;
use crate::sampler::Sampler;
use crate::scene::Scene;
use crate::spectrum::Spectrum;
use crate::stats;
use crate::Point2i;

stat_counter!("Integrator/Camera rays traced", n_camera_ray);
pub fn init_stats() {
    n_camera_ray::init();
}

pub fn render(
    scene: &Arc<Scene>,
    integrator: &mut dyn SamplerIntegrator,
    camera: &dyn Camera,
    num_threads: usize,
    sampler: &mut dyn Sampler,
    block_size: i32,
) -> Result<()> {
    integrator.preprocess(Arc::clone(scene), sampler);
    let sample_bounds = camera.get_film().get_sample_bounds();
    let sample_extent = sample_bounds.diagonal();
    let pixel_bounds = integrator.pixel_bounds();
    info!(
        "Rendering with sample_bounds = {}, pixel_bounds = {}",
        sample_bounds, pixel_bounds
    );
    let n_tiles = Point2i::new(
        (sample_extent.x + block_size - 1) / block_size,
        (sample_extent.y + block_size - 1) / block_size,
    );

    let num_blocks = n_tiles.x * n_tiles.y;
    info!("Rendering scene using {} threads", num_threads);
    let image_bounds =
        Bounds2i::from_points(&Point2i::new(0, 0), &Point2i::new(n_tiles.x, n_tiles.y));
    let tiles_iter = Arc::new(Mutex::new(image_bounds.into_iter()));
    let pb = indicatif::ProgressBar::new(num_blocks as _);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .progress_chars("=>-")
            .template("[{elapsed_precise}] [{wide_bar}] {percent}% [{pos}/{len}] {eta}"),
    );
    pb.tick();

    crossbeam::scope(|scope| {
        // We only want to use references to these in the thread, not move the structs themselves...
        let integrator = &integrator;
        let camera = &camera;
        let pb = &pb;

        // Spawn worker threads
        for _ in 0..num_threads {
            let mut sampler = sampler.box_clone();
            let tiles_iter = Arc::clone(&tiles_iter);
            scope.spawn(move |_| {
                loop {
                    let maybe_tile = {
                        let mut iter = tiles_iter.lock();
                        iter.next()
                    };
                    let tile = if let Some(t) = maybe_tile {
                        t
                    } else {
                        break;
                    };
                    // Render section of image corresponding to `tile`

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
                    let tile_bounds =
                        Bounds2i::from_points(&Point2i::new(x0, y0), &Point2i::new(x1, y1));
                    info!("Starting image tile {}", tile_bounds);

                    let mut film_tile = camera.get_film().get_film_tile(&tile_bounds);
                    for p in &tile_bounds {
                        sampler.start_pixel(p);

                        // Do this check after the start_pixel() call; this keeps
                        // the usage of RNG values from (most) Samplers that use
                        // RNGs consistent, which improves reproducability /
                        // debugging
                        if !pixel_bounds.inside_exclusive(&p) {
                            continue;
                        }

                        loop {
                            let alloc = arena.allocator();
                            let s = sampler.get_camera_sample(p);
                            let mut ray = camera.generate_ray_differential(&s);
                            ray.scale_differentials(1.0 / (sampler.spp() as f32).sqrt());
                            n_camera_ray::inc();
                            let mut sample_colour =
                                integrator.li(scene, &mut ray, sampler.as_mut(), &alloc, 0);
                            if sample_colour.has_nan() {
                                error!("Not-a-number radiance value returned for pixel {}, sample {}. Setting to black.", p, sampler.current_sample_number());
                                sample_colour = Spectrum::black();
                            }
                            if sample_colour.y() < -1e-5 {
                                error!("Negative luminance value, {}, returned for pixel {}, sample {}. Setting to black.", sample_colour.y(), p, sampler.current_sample_number());
                                sample_colour = Spectrum::black();
                            }
                            if sample_colour.y().is_infinite() {
                                error!("Infinite luminance value returned for pixel {}, sample {}. Setting to black.", p, sampler.current_sample_number());
                                sample_colour = Spectrum::black();
                            }
                            film_tile.add_sample(s.p_film, sample_colour);
                            if !sampler.start_next_sample() {
                                break;
                            }
                        }
                    }
                    camera.get_film().merge_film_tile(&film_tile);
                    pb.inc(1);
                }
                stats::report_stats();
            });
        }
    }).unwrap();
    pb.finish();

    camera.get_film().write_image()
}
