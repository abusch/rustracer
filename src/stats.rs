use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

static NUM_PRIMARY_RAYS: AtomicUsize = ATOMIC_USIZE_INIT;
static NUM_SECONDARY_RAYS: AtomicUsize = ATOMIC_USIZE_INIT;
static NUM_TRIANGLES: AtomicUsize = ATOMIC_USIZE_INIT;
static NUM_RAY_TRI_TEST: AtomicUsize = ATOMIC_USIZE_INIT;
static NUM_RAY_TRI_ISECT: AtomicUsize = ATOMIC_USIZE_INIT;
static NUM_FAST_BBOX_ISECT: AtomicUsize = ATOMIC_USIZE_INIT;

pub struct Stats {
    pub primary_rays: usize,
    pub secondary_rays: usize,
    pub triangles: usize,
    pub ray_triangle_tests: usize,
    pub ray_triangle_isect: usize,
    pub fast_bbox_isect: usize,
}

pub fn inc_primary_ray() {
    inc_counter(&NUM_PRIMARY_RAYS);
}
pub fn inc_secondary_ray() {
    inc_counter(&NUM_SECONDARY_RAYS);
}
pub fn inc_num_triangles() {
    inc_counter(&NUM_TRIANGLES);
}
pub fn inc_triangle_test() {
    inc_counter(&NUM_RAY_TRI_TEST);
}
pub fn inc_triangle_isect() {
    inc_counter(&NUM_RAY_TRI_ISECT);
}
pub fn inc_fast_bbox_isect() {
    inc_counter(&NUM_FAST_BBOX_ISECT);
}

pub fn get_stats() -> Stats {
    Stats {
        primary_rays: get_counter(&NUM_PRIMARY_RAYS),
        secondary_rays: get_counter(&NUM_SECONDARY_RAYS),
        triangles: get_counter(&NUM_TRIANGLES),
        ray_triangle_tests: get_counter(&NUM_RAY_TRI_TEST),
        ray_triangle_isect: get_counter(&NUM_RAY_TRI_ISECT),
        fast_bbox_isect: get_counter(&NUM_FAST_BBOX_ISECT),
    }
}

fn inc_counter(counter: &AtomicUsize) {
    counter.fetch_add(1, Ordering::SeqCst);
}

fn get_counter(counter: &AtomicUsize) -> usize {
    counter.load(Ordering::Relaxed)
}
