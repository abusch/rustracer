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


