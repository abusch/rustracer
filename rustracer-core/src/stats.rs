use std::collections::HashMap;

use parking_lot::Mutex;
use state::Storage;

#[derive(Default)]
pub struct StatAccumulator {
    counters: HashMap<String, u64>,
    memory_counters: HashMap<String, u64>,
    int_distribution_sums: HashMap<String, u64>,
    int_distribution_counts: HashMap<String, u64>,
    int_distribution_mins: HashMap<String, u64>,
    int_distribution_maxs: HashMap<String, u64>,
    percentages: HashMap<String, (u64, u64)>,
    ratios: HashMap<String, (u64, u64)>,
}

impl StatAccumulator {
    pub fn report_counter(&mut self, name: &str, value: u64) {
        let counter = self.counters.entry(name.to_owned()).or_insert(0);
        *counter += value;
    }

    pub fn report_memory_counter(&mut self, name: &str, value: u64) {
        let counter = self.memory_counters.entry(name.to_owned()).or_insert(0);
        *counter += value;
    }

    pub fn report_int_distribution(&mut self,
                                   name: &str,
                                   sum: u64,
                                   count: u64,
                                   min: u64,
                                   max: u64) {
        {
            let s = self.int_distribution_sums
                .entry(name.to_owned())
                .or_insert(0);
            *s += sum;
        }
        {
            let c = self.int_distribution_counts
                .entry(name.to_owned())
                .or_insert(0);
            *c += count;
        }
        {
            let m = self.int_distribution_mins
                .entry(name.to_owned())
                .or_insert(min);
            *m = u64::min(*m, min);
        }
        {
            let m = self.int_distribution_maxs
                .entry(name.to_owned())
                .or_insert(max);
            *m = u64::max(*m, max);
        }
    }

    pub fn report_percentage(&mut self, name: &str, num: u64, denom: u64) {
        let frac = self.percentages
            .entry(name.to_owned())
            .or_insert((0, 0));
        (*frac).0 += num;
        (*frac).1 += denom;
    }

    pub fn report_ratio(&mut self, name: &str, num: u64, denom: u64) {
        let frac = self.ratios.entry(name.to_owned()).or_insert((0, 0));
        (*frac).0 += num;
        (*frac).1 += denom;
    }

    pub fn print_stats(&self) {
        let mut to_print: HashMap<String, Vec<String>> = HashMap::new();
        println!("Statistics:");
        // Counters
        for (desc, value) in &self.counters {
            if *value == 0 {
                continue;
            }
            let (category, title) = self.get_category_and_title(desc);
            to_print
                .entry(category.to_owned())
                .or_insert(Vec::new())
                .push(format!("    {:<42}               {:12}", title, value));
        }
        // Memory counters
        for (desc, value) in &self.memory_counters {
            if *value == 0 {
                continue;
            }
            let (category, title) = self.get_category_and_title(desc);
            let kb = (*value as f64) / 1024.0;
            if kb < 1024.0 {
                to_print
                    .entry(category.to_owned())
                    .or_insert(Vec::new())
                    .push(format!("    {:<42}                  {:9.2} kiB", title, kb));
            } else {
                let mib = kb / 1024.0;
                if mib < 1024.0 {
                    to_print
                        .entry(category.to_owned())
                        .or_insert(Vec::new())
                        .push(format!("    {:<42}                  {:9.2} MiB", title, mib));
                } else {
                    let gib = mib / 1024.0;
                    to_print
                        .entry(category.to_owned())
                        .or_insert(Vec::new())
                        .push(format!("    {:<42}                  {:9.2} GiB", title, gib));
                }
            }
        }
        // Int distributions
        for (desc, sum) in &self.int_distribution_sums {
            let count = self.int_distribution_counts.get(desc).unwrap();
            if *count == 0 {
                continue;
            }
            let min = self.int_distribution_mins.get(desc).unwrap();
            let max = self.int_distribution_maxs.get(desc).unwrap();
            let (category, title) = self.get_category_and_title(desc);
            let avg = (*sum as f64) / (*count as f64);
            to_print
                .entry(category.to_owned())
                .or_insert(Vec::new())
                .push(format!("    {:<42}                      {:.3} avg [range {} - {}]",
                              title,
                              avg,
                              *min,
                              *max));

        }
        // Percentages
        for (desc, value) in &self.percentages {
            let (num, denom) = *value;
            if denom == 0 {
                continue;
            }
            let (category, title) = self.get_category_and_title(desc);
            to_print
                .entry(category.to_owned())
                .or_insert(Vec::new())
                .push(format!("    {:<42}{:12} / {:12} ({:.2}%)",
                              title,
                              num,
                              denom,
                              (num as f64 * 100.0) / (denom as f64)));
        }
        // Ratios
        for (desc, value) in &self.ratios {
            let (num, denom) = *value;
            if denom == 0 {
                continue;
            }
            let (category, title) = self.get_category_and_title(desc);
            to_print
                .entry(category.to_owned())
                .or_insert(Vec::new())
                .push(format!("    {:<42}{:12} / {:12} ({:.2}x)",
                              title,
                              num,
                              denom,
                              (num as f64) / (denom as f64)));
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

            #[allow(dead_code)]
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

#[macro_export]
macro_rules! stat_memory_counter(
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

            #[allow(dead_code)]
            #[inline(always)]
            pub fn add(a: u64) {
                let v = VALUE.get();
                v.set(v.get() + a);
            }

            pub fn report(acc: &mut StatAccumulator) {
                acc.report_memory_counter($d, VALUE.get().get());
            }
        }
    );
);

#[macro_export]
macro_rules! stat_int_distribution(
    ($d:expr, $x:ident) => (
        mod $x {
            use std::cell::Cell;
            use std::u64;
            use state::LocalStorage;
            use stats::StatAccumulator;

            static SUM: LocalStorage<Cell<u64>> = LocalStorage::new();
            static COUNT: LocalStorage<Cell<u64>> = LocalStorage::new();
            static MIN: LocalStorage<Cell<u64>> = LocalStorage::new();
            static MAX: LocalStorage<Cell<u64>> = LocalStorage::new();

            pub fn init() {
                SUM.set(|| Cell::new(0));
                COUNT.set(|| Cell::new(0));
                MIN.set(|| Cell::new(u64::MAX));
                MAX.set(|| Cell::new(u64::MIN));
                let mutex = $crate::stats::STAT_REPORTERS.get();
                let mut vec = mutex.lock();
                vec.push(Box::new(report));
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn report_value(v: u64) {
                let s = SUM.get();
                s.set(s.get() + v);
                let c = COUNT.get();
                c.set(c.get() + 1);
                let min = MIN.get();
                min.set(u64::min(min.get(), v));
                let max = MAX.get();
                max.set(u64::max(max.get(), v));
            }

            pub fn report(acc: &mut StatAccumulator) {
                acc.report_int_distribution(
                    $d,
                    SUM.get().get(),
                    COUNT.get().get(),
                    MIN.get().get(),
                    MAX.get().get());
            }
        }
    );
);

#[macro_export]
macro_rules! stat_percent(
    ($d:expr, $x:ident) => (
        mod $x {
            use std::cell::Cell;
            use state::LocalStorage;
            use stats::StatAccumulator;

            static NUM: LocalStorage<Cell<u64>> = LocalStorage::new();
            static DENOM: LocalStorage<Cell<u64>> = LocalStorage::new();

            pub fn init() {
                NUM.set(|| Cell::new(0));
                DENOM.set(|| Cell::new(0));
                let mutex = $crate::stats::STAT_REPORTERS.get();
                let mut vec = mutex.lock();
                vec.push(Box::new(report));
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn inc() {
                let v = NUM.get();
                v.set(v.get() + 1);
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn inc_total() {
                let v = DENOM.get();
                v.set(v.get() + 1);
            }

            pub fn report(acc: &mut StatAccumulator) {
                acc.report_percentage($d, NUM.get().get(), DENOM.get().get());
            }
        }
    );
);

#[macro_export]
macro_rules! stat_ratio(
    ($d:expr, $x:ident) => (
        mod $x {
            use std::cell::Cell;
            use state::LocalStorage;
            use stats::StatAccumulator;

            static NUM: LocalStorage<Cell<u64>> = LocalStorage::new();
            static DENOM: LocalStorage<Cell<u64>> = LocalStorage::new();

            pub fn init() {
                NUM.set(|| Cell::new(0));
                DENOM.set(|| Cell::new(0));
                let mutex = $crate::stats::STAT_REPORTERS.get();
                let mut vec = mutex.lock();
                vec.push(Box::new(report));
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn inc() {
                let v = NUM.get();
                v.set(v.get() + 1);
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn add(a: u64) {
                let v = NUM.get();
                v.set(v.get() + a);
            }

            #[allow(dead_code)]
            #[inline(always)]
            pub fn inc_total() {
                let v = DENOM.get();
                v.set(v.get() + 1);
            }

            pub fn report(acc: &mut StatAccumulator) {
                acc.report_ratio($d, NUM.get().get(), DENOM.get().get());
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
