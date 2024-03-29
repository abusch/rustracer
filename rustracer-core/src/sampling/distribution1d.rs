use crate::find_interval;

#[derive(Debug)]
pub struct Distribution1D {
    pub func: Vec<f32>,
    cdf: Vec<f32>,
    pub func_int: f32,
}

impl Distribution1D {
    pub fn new(f: &[f32]) -> Distribution1D {
        let n = f.len();
        let func = Vec::from(f);
        let mut cdf = vec![0.0; n + 1];
        // compute integral of step function at xi
        cdf[0] = 0.0;
        for i in 1..(n + 1) {
            cdf[i] = cdf[i - 1] + func[i - 1] / n as f32;
        }
        // transform step function integral into CDF
        let func_int = cdf[n];
        if func_int == 0.0 {
            cdf.iter_mut()
                .enumerate()
                .skip(1)
                .for_each(|(i, v)| *v = i as f32 / n as f32);
        // for i in 1..(n + 1) {
        //     cdf[i] = i as f32 / n as f32;
        // }
        } else {
            cdf.iter_mut().skip(1).for_each(|v| *v /= func_int);
            // for i in 1..(n + 1) {
            //     cdf[i] /= func_int;
            // }
        }

        Distribution1D {
            func,
            cdf,
            func_int,
        }
    }

    pub fn count(&self) -> usize {
        self.func.len()
    }

    pub fn sample_continuous(&self, u: f32) -> (f32, f32, usize) {
        // Find surrounding CDF segments and offset
        let offset = find_interval(self.cdf.len(), |i| self.cdf[i] <= u);
        // compute offset along CDF segment
        let mut du = u - self.cdf[offset];
        if self.cdf[offset + 1] - self.cdf[offset] > 0.0 {
            assert!(self.cdf[offset + 1] > self.cdf[offset]);
            du /= self.cdf[offset + 1] - self.cdf[offset];
        }
        // compute PDF for sampled offset
        let pdf = if self.func_int > 0.0 {
            self.func[offset] / self.func_int
        } else {
            0.0
        };

        // return x ∈ [0,1) corresponding to sample
        let x = (offset as f32 + du) / self.count() as f32;

        (x, pdf, offset)
    }

    pub fn sample_discrete(&self, u: f32) -> (usize, f32) {
        let offset = find_interval(self.cdf.len(), |i| self.cdf[i] <= u);
        let pdf = if self.func_int > 0.0 {
            self.func[offset] / (self.func_int * self.count() as f32)
        } else {
            0.0
        };

        (offset, pdf)
    }

    // TODO pdf_discrete
}

#[test]
fn test_discrete() {
    let func = [0.0, 1.0, 0.0, 3.0];
    let distrib = Distribution1D::new(&func[..]);

    assert_eq!(4, distrib.count());

    assert_eq!((1, 0.25), distrib.sample_discrete(0.0));
    assert_eq!((1, 0.25), distrib.sample_discrete(0.125));
    assert_eq!((1, 0.25), distrib.sample_discrete(0.24999));
    assert_eq!((3, 0.75), distrib.sample_discrete(0.250001));
    assert_eq!((3, 0.75), distrib.sample_discrete(0.625));
    assert_eq!((3, 0.75), distrib.sample_discrete(crate::ONE_MINUS_EPSILON));
    assert_eq!((3, 0.75), distrib.sample_discrete(1.0));
}
