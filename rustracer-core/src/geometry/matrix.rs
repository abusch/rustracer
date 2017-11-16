use std::ops::Mul;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Matrix4x4 {
    pub m: [[f32; 4]; 4],
}

impl Matrix4x4 {
    pub fn new() -> Matrix4x4 {
        Matrix4x4 {
            m: [[1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0]],
        }
    }

    pub fn from_elements(t00: f32,
                         t01: f32,
                         t02: f32,
                         t03: f32,
                         t10: f32,
                         t11: f32,
                         t12: f32,
                         t13: f32,
                         t20: f32,
                         t21: f32,
                         t22: f32,
                         t23: f32,
                         t30: f32,
                         t31: f32,
                         t32: f32,
                         t33: f32)
                         -> Matrix4x4 {
        Matrix4x4 {
            m: [[t00, t01, t02, t03],
                [t10, t11, t12, t13],
                [t20, t21, t22, t23],
                [t30, t31, t32, t33]],
        }
    }

    pub fn transpose(&self) -> Matrix4x4 {
        Matrix4x4::from_elements(self.m[0][0],
                                 self.m[1][0],
                                 self.m[2][0],
                                 self.m[3][0],
                                 self.m[0][1],
                                 self.m[1][1],
                                 self.m[2][1],
                                 self.m[3][1],
                                 self.m[0][2],
                                 self.m[1][2],
                                 self.m[2][2],
                                 self.m[3][2],
                                 self.m[0][3],
                                 self.m[1][3],
                                 self.m[2][3],
                                 self.m[3][3])
    }

    pub fn inverse(&self) -> Matrix4x4 {
        let mut indxc = [0usize; 4];
        let mut indxr = [0usize; 4];
        let mut ipiv = [0usize; 4];
        let mut minv = self.m;

        for i in 0..4 {
            let mut irow = 0;
            let mut icol = 0;
            let mut big = 0.0;

            // Choose pivot
            for j in 0..4 {
                if ipiv[j] != 1 {
                    for k in 0..4 {
                        if ipiv[k] == 0 {
                            if f32::abs(minv[j][k]) >= big {
                                big = f32::abs(minv[j][k]);
                                irow = j;
                                icol = k;
                            }
                        } else if ipiv[k] > 1 {
                            error!("Singular matrix in Matrix4x4::inverse()");
                        }
                    }
                }
            }
            ipiv[icol] += 1;
            // Swap rows `irow` and `icol` for pivot
            if irow != icol {
                for k in 0..4 {
                    let tmp = minv[irow][k];
                    minv[irow][k] = minv[icol][k];
                    minv[icol][k] = tmp;
                    // This doesn't work because I can't borrow minv mutably twice :(
                    // ::std::mem::swap(&mut minv[irow][k], &mut minv[icol][k]);
                }
            }
            indxr[i] = irow;
            indxc[i] = icol;
            if minv[icol][icol] == 0.0 {
                error!("Singular matrix in Matrix4x4::inverse()");
            }

            // Set `m[icol][icol]` to one by rscaling row `icol` appropriately
            let pivinv = 1.0 / minv[icol][icol];
            minv[icol][icol] = 1.0;
            for j in 0..4 {
                minv[icol][j] *= pivinv;
            }

            // Substract this row from others to zero out their columns
            for j in 0..4 {
                if j != icol {
                    let save = minv[j][icol];
                    minv[j][icol] = 0.0;
                    for k in 0..4 {
                        minv[j][k] -= minv[icol][k] * save;
                    }
                }
            }
        }

        // Swap columns to reflect permutation
        for j in (0..4).rev() {
            if indxr[j] != indxc[j] {
                for k in 0..4 {
                    let tmp = minv[k][indxr[j]];
                    minv[k][indxr[j]] = minv[k][indxc[j]];
                    minv[k][indxc[j]] = tmp;
                }
            }
        }

        Matrix4x4 { m: minv }
    }
}

impl Default for Matrix4x4 {
    fn default() -> Self {
        Matrix4x4::new()
    }
}

impl<'a, 'b> Mul<&'b Matrix4x4> for &'a Matrix4x4 {
    type Output = Matrix4x4;

    fn mul(self, m2: &'b Matrix4x4) -> Matrix4x4 {
        let mut r = Matrix4x4::new();
        for i in 0..4 {
            for j in 0..4 {
                r.m[i][j] = self.m[i][0] * m2.m[0][j] + self.m[i][1] * m2.m[1][j] +
                            self.m[i][2] * m2.m[2][j] +
                            self.m[i][3] * m2.m[3][j];
            }
        }
        r
    }
}
