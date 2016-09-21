use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

static NUM_PRIMARY_RAYS: AtomicUsize = ATOMIC_USIZE_INIT;
static NUM_SECONDARY_RAYS: AtomicUsize = ATOMIC_USIZE_INIT;

pub struct Stats {
    pub primary_rays: usize,
    pub secondary_rays: usize,
}

pub fn inc_primary_ray() {
    inc_counter(&NUM_PRIMARY_RAYS);
}
pub fn inc_secondary_ray() {
    inc_counter(&NUM_SECONDARY_RAYS);
}

pub fn get_stats() -> Stats {
    Stats {
        primary_rays: get_counter(&NUM_PRIMARY_RAYS),
        secondary_rays: get_counter(&NUM_SECONDARY_RAYS),
    }
}

fn inc_counter(counter: &AtomicUsize) {
    counter.fetch_add(1, Ordering::SeqCst);
}

fn get_counter(counter: &AtomicUsize) -> usize {
    counter.load(Ordering::Relaxed)
}
