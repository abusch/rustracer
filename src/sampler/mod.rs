use {Point2f, Point2i};
use camera::CameraSample;

pub mod zerotwosequence;
pub mod lowdiscrepancy;

pub trait Sampler {
    fn start_pixel(&mut self, p: &Point2i);
    // fn get_samples(&self, x: f32, y: f32, samples: &mut Vec<(f32, f32)>);
    fn get_1d(&mut self) -> f32;
    fn get_2d(&mut self) -> Point2f;
    fn get_camera_sample(&mut self, p_raster: &Point2i) -> CameraSample;
    fn request_1d_array(&mut self, n: usize);
    fn request_2d_array(&mut self, n: usize);
    fn round_count(&self, count: u32) -> u32;
    fn get_1d_array(&mut self, n: usize) -> &[f32];
    fn get_2d_array(&mut self, n: usize) -> &[Point2f];
    fn start_next_sample(&mut self) -> bool;
    fn reseed(&mut self, seed: u64);
}
