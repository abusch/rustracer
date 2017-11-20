extern crate ieee754;
extern crate rand;
extern crate rustracer_core as rt;

use rand::{Rng, SeedableRng, StdRng};
use ieee754::Ieee754;

use rt::efloat::EFloat;

const NUM_ITER: usize = 10_000;

/// Return an exponentially distributed floating-point value
#[cfg(test)]
fn get_float<T: Rng>(rng: &mut T, min_exp: f32, max_exp: f32) -> EFloat {
    let logu: f32 = rng.gen_range(min_exp, max_exp);
    let val = (10.0_f32).powf(logu);

    let err: f32 = match rng.gen_range(0, 4) {
        0 => 0.0,
        1 => {
            let ulp_err: u32 = rng.gen_range(0, 1024);
            let offset: f32 = Ieee754::from_bits(val.bits() + ulp_err);
            (offset - val).abs()
        }
        2 => {
            let ulp_err: u32 = rng.gen_range(0, 1024 * 1024);
            let offset: f32 = Ieee754::from_bits(val.bits() + ulp_err);
            (offset - val).abs()
        }
        3 => (4.0 * rng.next_f32()) * val.abs(),
        _ => panic!("should not happen"),
    };
    let sign = if rng.next_f32() < 0.5 { -1.0 } else { 1.0 };
    EFloat::new(sign * val, err)
}

#[cfg(test)]
fn get_precise<T: Rng>(ef: &EFloat, rng: &mut T) -> f64 {
    match rng.gen_range(0, 3) {
        0 => f64::from(ef.lower_bound()),
        1 => f64::from(ef.upper_bound()),
        2 => {
            let t = rng.next_f64();
            let p: f64 = (1.0 - t) * f64::from(ef.lower_bound()) + t * f64::from(ef.upper_bound());
            if p > f64::from(ef.upper_bound()) {
                f64::from(ef.upper_bound())
            } else if p < f64::from(ef.lower_bound()) {
                f64::from(ef.lower_bound())
            } else {
                p
            }
        }
        _ => panic!("should not happen"),
    }
}

#[test]
fn test_efloat_abs() {
    let mut rng = StdRng::from_seed(&[0]);
    for trial in 0..NUM_ITER {
        rng.reseed(&[trial]);
        let ef = get_float(&mut rng, -6.0, 6.0);
        let precise = get_precise(&ef, &mut rng);

        let result = ef.abs();
        let precise_result = precise.abs();

        assert!(precise_result >= f64::from(result.lower_bound()));
        assert!(precise_result <= f64::from(result.upper_bound()));
    }
}

#[test]
fn test_efloat_sqrt() {
    let mut rng = StdRng::from_seed(&[0]);
    for trial in 0..NUM_ITER {
        rng.reseed(&[trial]);
        let ef = get_float(&mut rng, -6.0, 6.0);
        let precise = get_precise(&ef, &mut rng);

        let result = ef.abs();
        let precise_result = precise.abs();

        assert!(precise_result >= f64::from(result.lower_bound()));
        assert!(precise_result <= f64::from(result.upper_bound()));
    }
}

#[test]
fn test_efloat_add() {
    let mut rng = StdRng::from_seed(&[0]);
    for trial in 0..NUM_ITER {
        rng.reseed(&[trial]);
        let a = get_float(&mut rng, -6.0, 6.0);
        let b = get_float(&mut rng, -6.0, 6.0);
        let ap = get_precise(&a, &mut rng);
        let bp = get_precise(&b, &mut rng);

        let result = a + b;
        let precise_result = ap + bp;

        assert!(precise_result >= f64::from(result.lower_bound()));
        assert!(precise_result <= f64::from(result.upper_bound()));
    }
}

#[test]
fn test_efloat_sub() {
    let mut rng = StdRng::from_seed(&[0]);
    for trial in 0..NUM_ITER {
        rng.reseed(&[trial]);
        let a = get_float(&mut rng, -6.0, 6.0);
        let b = get_float(&mut rng, -6.0, 6.0);
        let ap = get_precise(&a, &mut rng);
        let bp = get_precise(&b, &mut rng);

        let result = a - b;
        let precise_result = ap - bp;

        assert!(precise_result >= f64::from(result.lower_bound()));
        assert!(precise_result <= f64::from(result.upper_bound()));
    }
}

#[test]
fn test_efloat_mul() {
    let mut rng = StdRng::from_seed(&[0]);
    for trial in 0..NUM_ITER {
        rng.reseed(&[trial]);
        let a = get_float(&mut rng, -6.0, 6.0);
        let b = get_float(&mut rng, -6.0, 6.0);
        let ap = get_precise(&a, &mut rng);
        let bp = get_precise(&b, &mut rng);

        let result = a * b;
        let precise_result = ap * bp;

        assert!(precise_result >= f64::from(result.lower_bound()));
        assert!(precise_result <= f64::from(result.upper_bound()));
    }
}

#[test]
fn test_efloat_div() {
    let mut rng = StdRng::from_seed(&[0]);
    for trial in 0..NUM_ITER {
        rng.reseed(&[trial]);
        let a = get_float(&mut rng, -6.0, 6.0);
        let b = get_float(&mut rng, -6.0, 6.0);
        let ap = get_precise(&a, &mut rng);
        let bp = get_precise(&b, &mut rng);

        let result = a / b;
        let precise_result = ap / bp;

        assert!(precise_result >= f64::from(result.lower_bound()));
        assert!(precise_result <= f64::from(result.upper_bound()));
    }
}

#[test]
fn test_ieee754_next() {
    let neg_zero = -0.0f32;
    assert!(!neg_zero.prev().is_nan());
    assert!(!neg_zero.next().is_nan());
}
