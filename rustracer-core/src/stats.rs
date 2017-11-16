use std::ops::Add;
use std::cell::RefCell;

thread_local! {
    static NUM_CAMERA_RAYS: RefCell<u64> = RefCell::new(0);
    static NUM_PRIMARY_RAYS: RefCell<u64> = RefCell::new(0);
    static NUM_SECONDARY_RAYS: RefCell<u64> = RefCell::new(0);
    static NUM_TRIANGLES: RefCell<u64> = RefCell::new(0);
    static NUM_RAY_TRI_TEST: RefCell<u64> = RefCell::new(0);
    static NUM_RAY_TRI_ISECT: RefCell<u64> = RefCell::new(0);
    static NUM_FAST_BBOX_ISECT: RefCell<u64> = RefCell::new(0);
    static NUM_INTERSECTION_TEST: RefCell<u64> = RefCell::new(0);
    static NUM_SHADOW_TEST: RefCell<u64> = RefCell::new(0);
}

#[derive(Debug, Default)]
pub struct Stats {
    pub camera_rays: u64,
    pub primary_rays: u64,
    pub secondary_rays: u64,
    pub triangles: u64,
    pub ray_triangle_tests: u64,
    pub ray_triangle_isect: u64,
    pub fast_bbox_isect: u64,
    pub intersection_test: u64,
    pub shadow_test: u64,
}

pub fn inc_camera_ray() {
    NUM_CAMERA_RAYS.with(inc_counter);
}
pub fn inc_primary_ray() {
    NUM_PRIMARY_RAYS.with(inc_counter);
}
pub fn inc_secondary_ray() {
    NUM_SECONDARY_RAYS.with(inc_counter);
}
pub fn inc_num_triangles() {
    NUM_TRIANGLES.with(inc_counter);
}
pub fn inc_triangle_test() {
    NUM_RAY_TRI_TEST.with(inc_counter);
}
pub fn inc_triangle_isect() {
    NUM_RAY_TRI_ISECT.with(inc_counter);
}
pub fn inc_fast_bbox_isect() {
    NUM_FAST_BBOX_ISECT.with(inc_counter);
}
pub fn inc_intersection_test() {
    NUM_INTERSECTION_TEST.with(inc_counter);
}
pub fn inc_shadow_test() {
    NUM_SHADOW_TEST.with(inc_counter);
}

pub fn get_stats() -> Stats {
    Stats {
        camera_rays: NUM_CAMERA_RAYS.with(get_counter),
        primary_rays: NUM_PRIMARY_RAYS.with(get_counter),
        secondary_rays: NUM_SECONDARY_RAYS.with(get_counter),
        triangles: NUM_TRIANGLES.with(get_counter),
        ray_triangle_tests: NUM_RAY_TRI_TEST.with(get_counter),
        ray_triangle_isect: NUM_RAY_TRI_ISECT.with(get_counter),
        fast_bbox_isect: NUM_FAST_BBOX_ISECT.with(get_counter),
        intersection_test: NUM_INTERSECTION_TEST.with(get_counter),
        shadow_test: NUM_SHADOW_TEST.with(get_counter),
    }
}

fn inc_counter(counter: &RefCell<u64>) {
    *counter.borrow_mut() += 1;
}

fn get_counter(counter: &RefCell<u64>) -> u64 {
    *counter.borrow()
}

impl Add<Stats> for Stats {
    type Output = Stats;

    fn add(self, rhs: Stats) -> Stats {
        Stats {
            camera_rays: self.camera_rays + rhs.camera_rays,
            primary_rays: self.primary_rays + rhs.primary_rays,
            secondary_rays: self.secondary_rays + rhs.secondary_rays,
            triangles: self.triangles + rhs.triangles,
            ray_triangle_tests: self.ray_triangle_tests + rhs.ray_triangle_tests,
            ray_triangle_isect: self.ray_triangle_isect + rhs.ray_triangle_isect,
            fast_bbox_isect: self.fast_bbox_isect + rhs.fast_bbox_isect,
            intersection_test: self.intersection_test + rhs.intersection_test,
            shadow_test: self.shadow_test + rhs.shadow_test,
        }
    }
}
