use Point2i;
use bounds::Bounds2i;
use super::lowdiscrepancy::sobol_interval_to_index;

pub struct SobolSampler {
    dimension: usize,
    interval_sample_index: u64,
    array_start_dim: usize,
    array_end_dim: usize,
    sample_bounds: Bounds2i,
    resolution: u32,
    log2_resolution: u32,
}

impl SobolSampler {
    fn get_index_for_sample(&self, sample_num: u64) -> u64 {
        sobol_interval_to_index(
            self.log2_resolution,
            sample_num,
            &Point2i::new(self.current_pixel - sample_bounds.p_min),
        )
    }
    fn sample_dimension(&self, index: u64, dim: usize) -> f32 {
        0.0
    }
}
