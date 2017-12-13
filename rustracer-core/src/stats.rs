use std::collections::HashMap;

use parking_lot::Mutex;
use state::Storage;

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
