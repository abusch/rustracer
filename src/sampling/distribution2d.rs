use ::Point2f;
use sampling::Distribution1D;

pub struct Distribution2D {
    p_conditional_v: Vec<Distribution1D>,
    p_marginal: Distribution1D,
}

impl Distribution2D {
    pub fn new(func: &[f32], nu: usize, nv: usize) -> Distribution2D {
        let p_conditional_v = Vec::new();
        for v in 0..nv {
            // compute conditional sampling distribution for v_tilde
            p_conditional_v.push(Distribution1D::new(&func[v * nu..v * (nu + 1)]));
        }
        // compute marginal sampling distribution p[v_tilde]
        let marginal_func = Vec::with_capacity(nv);
        for v in 0..nv {
            marginal_func.push(p_conditional_v[v].func_int)
        }

        Distribution2D {
            p_conditional_v: p_conditional_v,
            p_marginal: Distribution1D::new(&marginal_func[..]),
        }
    }

    pub fn sample_continuous(&self, u: &Point2f) -> (Point2f, f32) {
        let (d_0, pdf_0, v) = self.p_marginal.sample_continuous(u[0]);
        let (d_1, pdf_1, _) = self.p_conditional_v[v].sample_continuous(u[1]);

        (Point2f::new(d_0, d_1), pdf_0 * pdf_1)
    }
}
