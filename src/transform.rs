use std::ops::Mul;
use na::{self, Matrix4, Matrix2, Similarity3, U3};

use {Vector2f, Vector3f, Point3f, gamma};

#[derive(Debug,Clone)]
pub struct Transform {
    pub m: Matrix4<f32>,
    pub m_inv: Matrix4<f32>,
}

impl Transform {
    pub fn new(t: Vector3f, r: Vector3f, s: f32) -> Transform {
        Transform::from_similarity(&Similarity3::new(t, r, s))
    }

    pub fn from_similarity(s: &Similarity3<f32>) -> Transform {
        Transform {
            m: s.to_homogeneous(),
            m_inv: s.inverse().to_homogeneous(),
        }
    }

    pub fn rot_x(angle: f32) -> Transform {
        Transform::new(na::zero(), Vector3f::x() * angle.to_radians(), 1.0)
    }

    pub fn rot_y(angle: f32) -> Transform {
        Transform::new(na::zero(), Vector3f::y() * angle.to_radians(), 1.0)
    }

    pub fn rot_z(angle: f32) -> Transform {
        Transform::new(na::zero(), Vector3f::z() * angle.to_radians(), 1.0)
    }

    pub fn rot(ax: f32, ay: f32, az: f32) -> Transform {
        Transform::new(na::zero(), Vector3f::new(ax, ay, az), 1.0)
    }

    pub fn translate_x(t: f32) -> Transform {
        Transform::new(Vector3f::x() * t, na::zero(), 1.0)
    }

    pub fn translate_y(t: f32) -> Transform {
        Transform::new(Vector3f::y() * t, na::zero(), 1.0)
    }

    pub fn translate_z(t: f32) -> Transform {
        Transform::new(Vector3f::z() * t, na::zero(), 1.0)
    }

    pub fn inverse(&self) -> Self {
        Transform {
            m: self.m_inv,
            m_inv: self.m,
        }
    }

    /// Transform the given point using the given transformation and also return a vector of the
    /// absolute error introduced for each coordinate.
    pub fn transform_point(&self, p: &Point3f) -> (Point3f, Vector3f) {
        let (x, y, z) = (p.x, p.y, p.z);
        let tp = self * p;
        let m = self.m;
        let x_abs_sum = (m[(0, 0)] * x).abs() + (m[(0, 1)] * y).abs() + (m[(0, 2)] * z).abs() +
                        m[(0, 3)].abs();
        let y_abs_sum = (m[(1, 0)] * x).abs() + (m[(1, 1)] * y).abs() + (m[(1, 2)] * z).abs() +
                        m[(1, 3)].abs();
        let z_abs_sum = (m[(2, 0)] * x).abs() + (m[(2, 1)] * y).abs() + (m[(2, 2)] * z).abs() +
                        m[(2, 3)].abs();
        let p_err = gamma(3) * Vector3f::new(x_abs_sum, y_abs_sum, z_abs_sum);

        (tp, p_err)
    }

    pub fn transform_point_with_error(&self,
                                      p: &Point3f,
                                      p_error: &Vector3f)
                                      -> (Point3f, Vector3f) {
        let (x, y, z) = (p.x, p.y, p.z);
        let tp = self * p;
        let m = self.m;
        let x_abs_err = (gamma(3) + 1.0) *
                        ((m[(0, 0)] * p_error.x).abs() + (m[(0, 1)] * p_error.y).abs() +
                         (m[(0, 2)] * p_error.z).abs()) +
                        gamma(3) *
                        ((m[(0, 0)] * x).abs() + (m[(0, 1)] * y).abs() + (m[(0, 2)] * z).abs() +
                         m[(0, 3)].abs());
        let y_abs_err = (gamma(3) + 1.0) *
                        ((m[(1, 0)] * p_error.x).abs() + (m[(1, 1)] * p_error.y).abs() +
                         (m[(1, 2)] * p_error.z).abs()) +
                        gamma(3) *
                        ((m[(1, 0)] * x).abs() + (m[(1, 1)] * y).abs() + (m[(1, 2)] * z).abs() +
                         m[(1, 3)].abs());
        let z_abs_err = (gamma(3) + 1.0) *
                        ((m[(2, 0)] * p_error.x).abs() + (m[(2, 1)] * p_error.y).abs() +
                         (m[(2, 2)] * p_error.z).abs()) +
                        gamma(3) *
                        ((m[(2, 0)] * x).abs() + (m[(2, 1)] * y).abs() + (m[(2, 2)] * z).abs() +
                         m[(2, 3)].abs());
        let p_err = Vector3f::new(x_abs_err, y_abs_err, z_abs_err);

        (tp, p_err)
    }

    /// Transform the given point using the given transformation and also return a vector of the
    /// absolute error introduced for each coordinate.
    pub fn transform_vector(&self, v: &Vector3f) -> (Vector3f, Vector3f) {
        let (x, y, z) = (v.x, v.y, v.z);
        let tv = self * v;
        let m = self.m;
        let x_abs_sum = na::abs(&(m[(0, 0)] * x)) + na::abs(&(m[(0, 1)] * y)) +
                        na::abs(&(m[(0, 2)] * z)) + na::abs(&m[(0, 3)]);
        let y_abs_sum = na::abs(&(m[(1, 0)] * x)) + na::abs(&(m[(1, 1)] * y)) +
                        na::abs(&(m[(1, 2)] * z)) + na::abs(&m[(1, 3)]);
        let z_abs_sum = na::abs(&(m[(2, 0)] * x)) + na::abs(&(m[(2, 1)] * y)) +
                        na::abs(&(m[(2, 2)] * z)) + na::abs(&m[(2, 3)]);
        let v_err = gamma(3) * Vector3f::new(x_abs_sum, y_abs_sum, z_abs_sum);

        (tv, v_err)
    }

    pub fn transform_normal(&self, normal: &Vector3f) -> Vector3f {
        let m = self.m_inv.transpose();
        m.fixed_slice::<U3, U3>(0, 0) * *normal
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            m: na::one(),
            m_inv: na::one(),
        }
    }
}

impl<'a, 'b> Mul<&'a Point3f> for &'b Transform {
    type Output = Point3f;

    fn mul(self, p: &'a Point3f) -> Point3f {
        let x = p.x;
        let y = p.y;
        let z = p.z;
        let xp = self.m[(0, 0)] * x + self.m[(0, 1)] * y + self.m[(0, 2)] * z + self.m[(0, 3)];
        let yp = self.m[(1, 0)] * x + self.m[(1, 1)] * y + self.m[(1, 2)] * z + self.m[(1, 3)];
        let zp = self.m[(2, 0)] * x + self.m[(2, 1)] * y + self.m[(2, 2)] * z + self.m[(2, 3)];
        let wp = self.m[(3, 0)] * x + self.m[(3, 1)] * y + self.m[(3, 2)] * z + self.m[(3, 3)];

        assert!(wp != 0.0);

        if wp == 1.0 {
            Point3f::new(xp, yp, zp)
        } else {
            Point3f::new(xp, yp, zp) / wp
        }
    }
}

impl<'a, 'b> Mul<&'a Vector3f> for &'b Transform {
    type Output = Vector3f;

    fn mul(self, v: &'a Vector3f) -> Vector3f {
        let x = v.x;
        let y = v.y;
        let z = v.z;

        Vector3f::new(self.m[(0, 0)] * x + self.m[(0, 1)] * y + self.m[(0, 2)] * z,
                      self.m[(1, 0)] * x + self.m[(1, 1)] * y + self.m[(1, 2)] * z,
                      self.m[(2, 0)] * x + self.m[(2, 1)] * y + self.m[(2, 2)] * z)

    }
}

impl<'a, 'b> Mul<&'a Transform> for &'b Transform {
    type Output = Transform;

    fn mul(self, t: &'a Transform) -> Transform {
        Transform {
            m: self.m * t.m,
            m_inv: t.m_inv * self.m_inv,
        }
    }
}

#[allow(non_snake_case)]
pub fn solve_linear_system2x2(A: &Matrix2<f32>, B: &Vector2f) -> Option<(f32, f32)> {
    let det = A.determinant();
    if det.abs() < 1e-10 {
        return None;
    }
    let x0 = (A[(1, 1)] * B[0] - A[(0, 1)] * B[1]) / det;
    let x1 = (A[(0, 0)] * B[1] - A[(1, 0)] * B[0]) / det;

    if x0.is_nan() || x1.is_nan() {
        return None;
    }
    return Some((x0, x1));
}


#[test]
fn test_normal_transform() {
    let t = Transform::new(Vector3f::new(0.0, 0.0, 0.0),
                           Vector3f::new(4.0, 5.0, 6.0),
                           4.0);
    let t_inv = t.inverse();

    let v = Vector3f::x();
    let n = Vector3f::y();
    println!("v = {}, n = {}", v, n);
    assert_eq!(v.dot(&n), 0.0);

    let v2 = &t * &v;
    let n2 = t_inv.transform_normal(&n);
    println!("v = {}, n = {}", v2, n2);
    relative_eq!(v2.dot(&n2), 0.0);
}
