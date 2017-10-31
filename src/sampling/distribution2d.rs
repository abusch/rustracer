use {clamp, Point2f};
use sampling::Distribution1D;

#[derive(Debug)]
pub struct Distribution2D {
    p_conditional_v: Vec<Distribution1D>,
    p_marginal: Distribution1D,
}

impl Distribution2D {
    pub fn new(func: &[f32], nu: usize, nv: usize) -> Distribution2D {
        let mut p_conditional_v = Vec::with_capacity(nv);
        for v in 0..nv {
            // compute conditional sampling distribution for v_tilde
            p_conditional_v.push(Distribution1D::new(&func[v * nu..(v + 1) * nu]));
        }
        // compute marginal sampling distribution p[v_tilde]
        let mut marginal_func = Vec::with_capacity(nv);
        for v in 0..nv {
            marginal_func.push(p_conditional_v[v].func_int)
        }

        Distribution2D {
            p_conditional_v: p_conditional_v,
            p_marginal: Distribution1D::new(&marginal_func[..]),
        }
    }

    pub fn sample_continuous(&self, u: &Point2f) -> (Point2f, f32) {
        let (d_1, pdf_1, v) = self.p_marginal.sample_continuous(u[1]);
        let (d_0, pdf_0, _) = self.p_conditional_v[v].sample_continuous(u[0]);

        (Point2f::new(d_0, d_1), pdf_0 * pdf_1)
    }

    pub fn pdf(&self, p: &Point2f) -> f32 {
        let iu = clamp(
            p[0] as usize * self.p_conditional_v[0].count(),
            0,
            self.p_conditional_v[0].count() - 1,
        );
        let iv = clamp(
            p[1] as usize * self.p_marginal.count(),
            0,
            self.p_marginal.count() - 1,
        );

        self.p_conditional_v[iv].func[iu] / self.p_marginal.func_int
    }
}
