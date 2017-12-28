use {Point2f, Point2i};
use camera::CameraSample;

pub mod zerotwosequence;
pub mod lowdiscrepancy;

pub trait Sampler: Send + Sync {
    fn start_pixel(&mut self, p: &Point2i);
    fn get_1d(&mut self) -> f32;
    fn get_2d(&mut self) -> Point2f;
    fn get_camera_sample(&mut self, p_raster: &Point2i) -> CameraSample;
    fn request_1d_array(&mut self, n: usize);
    fn request_2d_array(&mut self, n: usize);
    fn round_count(&self, count: usize) -> usize;
    fn get_1d_array(&mut self, n: usize) -> Option<&[f32]>;
    fn get_2d_array(&mut self, n: usize) -> Option<&[Point2f]>;
    fn start_next_sample(&mut self) -> bool;
    fn reseed(&mut self, seed: u64);
    fn spp(&self) -> usize;
    fn box_clone(&self) -> Box<Sampler>;
    fn current_sample_number(&self) -> usize;
}

impl Clone for Box<Sampler> {
    fn clone(&self) -> Box<Sampler> {
        self.box_clone()
    }
}
