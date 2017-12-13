use std::ops::Add;
use std::cell::RefCell;
use std::collections::HashMap;

use parking_lot::Mutex;
use state::Storage;


thread_local! {
    static NUM_TRIANGLES: RefCell<u64> = RefCell::new(0);
    static NUM_RAY_TRI_TEST: RefCell<u64> = RefCell::new(0);
    static NUM_RAY_TRI_ISECT: RefCell<u64> = RefCell::new(0);
    static NUM_FAST_BBOX_ISECT: RefCell<u64> = RefCell::new(0);
}

#[derive(Default)]
pub struct StatAccumulator {
    counters: HashMap<String, u64>,
}

impl StatAccumulator {
    pub fn report_counter(&mut self, name: &str, value: u64) {
        let counter = self.counters.entry(name.to_owned()).or_insert(0);
        *counter += value;
    }

    pub fn print_stats(&self) {
        let mut to_print: HashMap<String, Vec<String>> = HashMap::new();
        println!("Statistics:");
        for (desc, value) in &self.counters {
            if *value == 0 {
                continue;
            }
            let (category, title) = self.get_category_and_title(desc);
            to_print
                .entry(category.to_owned())
                .or_insert(Vec::new())
                .push(format!("    {:<42}{:12}", title, value));
        }

        for (category, stats) in &to_print {
            println!("  {}", category);
            for s in stats {
                println!("{}", s);
            }
        }
    }

    fn get_category_and_title<'a>(&self, s: &'a str) -> (&'a str, &'a str) {
        let v: Vec<&'a str> = s.split('/').collect();
        if v.len() > 1 { (v[0], v[1]) } else { ("", s) }
    }
}

type StatReporterFn = Box<Fn(&mut StatAccumulator) + Send>;
pub static STAT_REPORTERS: Storage<Mutex<Vec<StatReporterFn>>> = Storage::new();
pub static STAT_ACCUMULATOR: Storage<Mutex<StatAccumulator>> = Storage::new();

pub struct StatRegisterer {
    // pub
}

#[macro_export]
macro_rules! stat_counter(
    ($d:expr, $x:ident) => (
        mod $x {
            use std::cell::Cell;
            use state::LocalStorage;
            use stats::StatAccumulator;

            static VALUE: LocalStorage<Cell<u64>> = LocalStorage::new();

            pub fn init() {
                VALUE.set(|| Cell::new(0));
                let mutex = $crate::stats::STAT_REPORTERS.get();
                let mut vec = mutex.lock();
                vec.push(Box::new(report));
            }

            #[inline(always)]
            pub fn inc() {
                let v = VALUE.get();
                v.set(v.get() + 1);
            }

            pub fn report(acc: &mut StatAccumulator) {
                acc.report_counter($d, VALUE.get().get());
            }
        }
    );
);

#[derive(Debug, Default)]
pub struct Stats {
    pub triangles: u64,
    pub ray_triangle_tests: u64,
    pub ray_triangle_isect: u64,
    pub fast_bbox_isect: u64,
}

pub fn init_stats() {
    STAT_REPORTERS.set(Mutex::new(Vec::new()));
    STAT_ACCUMULATOR.set(Mutex::new(StatAccumulator::default()));
}

pub fn report_stats() {
    let vec = STAT_REPORTERS.get().lock();
    let mut acc = STAT_ACCUMULATOR.get().lock();
    for f in &(*vec) {
        f(&mut acc);
    }
}

pub fn print_stats() {
    let acc = STAT_ACCUMULATOR.get().lock();
    (*acc).print_stats();
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

pub fn get_stats() -> Stats {
    Stats {
        triangles: NUM_TRIANGLES.with(get_counter),
        ray_triangle_tests: NUM_RAY_TRI_TEST.with(get_counter),
        ray_triangle_isect: NUM_RAY_TRI_ISECT.with(get_counter),
        fast_bbox_isect: NUM_FAST_BBOX_ISECT.with(get_counter),
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
            triangles: self.triangles + rhs.triangles,
            ray_triangle_tests: self.ray_triangle_tests + rhs.ray_triangle_tests,
            ray_triangle_isect: self.ray_triangle_isect + rhs.ray_triangle_isect,
            fast_bbox_isect: self.fast_bbox_isect + rhs.fast_bbox_isect,
        }
    }
}
