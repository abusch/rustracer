use ::find_interval;

pub struct Distribution1D {
    func: Vec<f32>,
    cdf: Vec<f32>,
    pub func_int: f32,
}

impl Distribution1D {
    pub fn new(f: &[f32]) -> Distribution1D {
        let n = f.len();
        let func = Vec::from(f);
        let mut cdf = Vec::with_capacity(n + 1);
        // compute integral of step function at xi
        cdf[0] = 0.0;
        for i in 0..n + 1 {
            cdf[i] = cdf[i - 1] + func[i - 1] / n as f32;
        }
        // transform step function integral into CDF
        let func_int = cdf[n];
        if func_int == 0.0 {
            for i in 0..n + 1 {
                cdf[i] = i as f32 / n as f32;
            }
        } else {
            for i in 0..n + 1 {
                cdf[i] /= func_int;
            }
        }

        Distribution1D {
            cdf: cdf,
            func: func,
            func_int: func_int,
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
            du /= self.cdf[offset + 1] - self.cdf[offset];
        }
        // compute PDF for sampled offset
        let pdf = self.func[offset] / self.func_int;

        // return x ∈ [0,1) corresponding to sample
        let x = (offset as f32 + du) / self.count() as f32;

        (x, pdf, offset)
    }

    // TODO sample_discrete
    // TODO pdf_discrete
}
