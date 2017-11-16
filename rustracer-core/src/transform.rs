use std::ops::Mul;

use {gamma, Normal3f, Point3f, Vector2f, Vector3f};
use bounds::Bounds3f;
use geometry::Matrix4x4;

#[derive(Debug, Clone, Default)]
pub struct Transform {
    pub m: Matrix4x4,
    pub m_inv: Matrix4x4,
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            m: Matrix4x4::new(),
            m_inv: Matrix4x4::new(),
        }
    }

    pub fn from_matrix(m: Matrix4x4) -> Transform {
        Transform {
            m_inv: m.inverse(),
            m: m,
        }
    }

    pub fn rotate(theta: f32, axis: Vector3f) -> Transform {
        let a = axis.normalize();
        let sin_theta = f32::sin(theta.to_radians());
        let cos_theta = f32::cos(theta.to_radians());
        let mut m = Matrix4x4::new();
        // Compute rotation of first basis vector
        m.m[0][0] = a.x * a.x + (1.0 - a.x * a.x) * cos_theta;
        m.m[0][1] = a.x * a.y * (1.0 - cos_theta) - a.z * sin_theta;
        m.m[0][2] = a.x * a.z * (1.0 - cos_theta) + a.y * sin_theta;
        m.m[0][3] = 0.0;

        // Compute rotations of second and third basis vectors
        m.m[1][0] = a.x * a.y * (1.0 - cos_theta) + a.z * sin_theta;
        m.m[1][1] = a.y * a.y + (1.0 - a.y * a.y) * cos_theta;
        m.m[1][2] = a.y * a.z * (1.0 - cos_theta) - a.x * sin_theta;
        m.m[1][3] = 0.0;

        m.m[2][0] = a.x * a.z * (1.0 - cos_theta) - a.y * sin_theta;
        m.m[2][1] = a.y * a.z * (1.0 - cos_theta) + a.x * sin_theta;
        m.m[2][2] = a.z * a.z + (1.0 - a.z * a.z) * cos_theta;
        m.m[2][3] = 0.0;

        Transform {
            m: m,
            m_inv: m.transpose(),
        }
    }

    pub fn rot_x(angle: f32) -> Transform {
        Transform::rotate(angle, Vector3f::x())
    }

    pub fn rot_y(angle: f32) -> Transform {
        Transform::rotate(angle, Vector3f::y())
    }

    pub fn rot_z(angle: f32) -> Transform {
        Transform::rotate(angle, Vector3f::z())
    }

    pub fn translate(delta: &Vector3f) -> Transform {
        let m = Matrix4x4::from_elements(1.0,
                                         0.0,
                                         0.0,
                                         delta.x,
                                         0.0,
                                         1.0,
                                         0.0,
                                         delta.y,
                                         0.0,
                                         0.0,
                                         1.0,
                                         delta.z,
                                         0.0,
                                         0.0,
                                         0.0,
                                         1.0);
        let minv = Matrix4x4::from_elements(1.0,
                                            0.0,
                                            0.0,
                                            -delta.x,
                                            0.0,
                                            1.0,
                                            0.0,
                                            -delta.y,
                                            0.0,
                                            0.0,
                                            1.0,
                                            -delta.z,
                                            0.0,
                                            0.0,
                                            0.0,
                                            1.0);
        Transform { m: m, m_inv: minv }
    }

    pub fn translate_x(t: f32) -> Transform {
        Transform::translate(&(Vector3f::x() * t))
    }

    pub fn translate_y(t: f32) -> Transform {
        Transform::translate(&(Vector3f::y() * t))
    }

    pub fn translate_z(t: f32) -> Transform {
        Transform::translate(&(Vector3f::z() * t))
    }

    pub fn scale(sx: f32, sy: f32, sz: f32) -> Transform {
        let m = Matrix4x4::from_elements(sx,
                                         0.0,
                                         0.0,
                                         0.0,
                                         0.0,
                                         sy,
                                         0.0,
                                         0.0,
                                         0.0,
                                         0.0,
                                         sz,
                                         0.0,
                                         0.0,
                                         0.0,
                                         0.0,
                                         1.0);
        let minv = Matrix4x4::from_elements(1.0 / sx,
                                            0.0,
                                            0.0,
                                            0.0,
                                            0.0,
                                            1.0 / sy,
                                            0.0,
                                            0.0,
                                            0.0,
                                            0.0,
                                            1.0 / sz,
                                            0.0,
                                            0.0,
                                            0.0,
                                            0.0,
                                            1.0);
        Transform { m: m, m_inv: minv }
    }

    pub fn look_at(pos: &Point3f, look: &Point3f, up: &Vector3f) -> Transform {
        let mut camera_to_world = Matrix4x4::new();
        // Initialize fourth column of viewing matrix
        camera_to_world.m[0][3] = pos.x;
        camera_to_world.m[1][3] = pos.y;
        camera_to_world.m[2][3] = pos.z;
        camera_to_world.m[3][3] = 1.0;

        // Initialize first three columns of viewing matrix
        let dir = (*look - *pos).normalize();
        if up.normalize().cross(&dir).length() == 0.0 {
            error!("\"up\" vector {} and viewing direction {} passed to LookAt are pointing in the same direction.  Using the identity transformation.",
                   up,
                   dir);
            return Transform::new();
        }
        let left = up.normalize().cross(&dir).normalize();
        let new_up = dir.cross(&left);
        camera_to_world.m[0][0] = left.x;
        camera_to_world.m[1][0] = left.y;
        camera_to_world.m[2][0] = left.z;
        camera_to_world.m[3][0] = 0.0;
        camera_to_world.m[0][1] = new_up.x;
        camera_to_world.m[1][1] = new_up.y;
        camera_to_world.m[2][1] = new_up.z;
        camera_to_world.m[3][1] = 0.0;
        camera_to_world.m[0][2] = dir.x;
        camera_to_world.m[1][2] = dir.y;
        camera_to_world.m[2][2] = dir.z;
        camera_to_world.m[3][2] = 0.0;

        Transform {
            m: camera_to_world.inverse(),
            m_inv: camera_to_world,
        }
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn perspective(fov: f32, n: f32, f: f32) -> Transform {
        let persp = Matrix4x4::from_elements(1.0, 0.0, 0.0, 0.0,
                                             0.0, 1.0, 0.0, 0.0,
                                             0.0, 0.0, f / (f - n), -f * n / (f - n),
                                             0.0, 0.0, 1.0, 0.0);
        let inv_tan_ang = 1.0 / f32::tan(fov.to_radians() / 2.0);
        &Transform::scale(inv_tan_ang, inv_tan_ang, 1.0) * &Transform::from_matrix(persp)
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
        let m = self.m.m;
        let x_abs_sum = (m[0][0] * x).abs() + (m[0][1] * y).abs() + (m[0][2] * z).abs() +
                        m[0][3].abs();
        let y_abs_sum = (m[1][0] * x).abs() + (m[1][1] * y).abs() + (m[1][2] * z).abs() +
                        m[1][3].abs();
        let z_abs_sum = (m[2][0] * x).abs() + (m[2][1] * y).abs() + (m[2][2] * z).abs() +
                        m[2][3].abs();
        let p_err = gamma(3) * Vector3f::new(x_abs_sum, y_abs_sum, z_abs_sum);

        (tp, p_err)
    }

    pub fn transform_point_with_error(&self,
                                      p: &Point3f,
                                      p_error: &Vector3f)
                                      -> (Point3f, Vector3f) {
        let (x, y, z) = (p.x, p.y, p.z);
        let tp = self * p;
        let m = self.m.m;
        let x_abs_err = (gamma(3) + 1.0) *
                        ((m[0][0] * p_error.x).abs() + (m[0][1] * p_error.y).abs() +
                         (m[0][2] * p_error.z).abs()) +
                        gamma(3) *
                        ((m[0][0] * x).abs() + (m[0][1] * y).abs() + (m[0][2] * z).abs() +
                         m[0][3].abs());
        let y_abs_err = (gamma(3) + 1.0) *
                        ((m[1][0] * p_error.x).abs() + (m[1][1] * p_error.y).abs() +
                         (m[1][2] * p_error.z).abs()) +
                        gamma(3) *
                        ((m[1][0] * x).abs() + (m[1][1] * y).abs() + (m[1][2] * z).abs() +
                         m[1][3].abs());
        let z_abs_err = (gamma(3) + 1.0) *
                        ((m[2][0] * p_error.x).abs() + (m[2][1] * p_error.y).abs() +
                         (m[2][2] * p_error.z).abs()) +
                        gamma(3) *
                        ((m[2][0] * x).abs() + (m[2][1] * y).abs() + (m[2][2] * z).abs() +
                         m[2][3].abs());
        let p_err = Vector3f::new(x_abs_err, y_abs_err, z_abs_err);

        (tp, p_err)
    }

    /// Transform the given point using the given transformation and also return a vector of the
    /// absolute error introduced for each coordinate.
    pub fn transform_vector(&self, v: &Vector3f) -> (Vector3f, Vector3f) {
        let (x, y, z) = (v.x, v.y, v.z);
        let tv = self * v;
        let m = self.m.m;
        let x_abs_sum = f32::abs(m[0][0] * x) + f32::abs(m[0][1] * y) + f32::abs(m[0][2] * z) +
                        f32::abs(m[0][3]);
        let y_abs_sum = f32::abs(m[1][0] * x) + f32::abs(m[1][1] * y) + f32::abs(m[1][2] * z) +
                        f32::abs(m[1][3]);
        let z_abs_sum = f32::abs(m[2][0] * x) + f32::abs(m[2][1] * y) + f32::abs(m[2][2] * z) +
                        f32::abs(m[2][3]);
        let v_err = gamma(3) * Vector3f::new(x_abs_sum, y_abs_sum, z_abs_sum);

        (tv, v_err)
    }

    pub fn transform_normal(&self, normal: &Normal3f) -> Normal3f {
        let (x, y, z) = (normal.x, normal.y, normal.z);
        let m = self.m_inv.m;

        Normal3f::new(m[0][0] * x + m[1][0] * y + m[2][0] * z,
                      m[0][1] * x + m[1][1] * y + m[2][1] * z,
                      m[0][2] * x + m[1][2] * y + m[2][2] * z)
    }

    pub fn swaps_handedness(&self) -> bool {
        let m = self.m.m;
        let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) -
                  m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) +
                  m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
        det < 0.0
    }
}

impl<'a, 'b> Mul<&'a Point3f> for &'b Transform {
    type Output = Point3f;

    fn mul(self, p: &'a Point3f) -> Point3f {
        let x = p.x;
        let y = p.y;
        let z = p.z;
        let m = self.m.m;
        let xp = m[0][0] * x + m[0][1] * y + m[0][2] * z + m[0][3];
        let yp = m[1][0] * x + m[1][1] * y + m[1][2] * z + m[1][3];
        let zp = m[2][0] * x + m[2][1] * y + m[2][2] * z + m[2][3];
        let wp = m[3][0] * x + m[3][1] * y + m[3][2] * z + m[3][3];

        assert_ne!(wp, 0.0);

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
        let m = self.m.m;

        Vector3f::new(m[0][0] * x + m[0][1] * y + m[0][2] * z,
                      m[1][0] * x + m[1][1] * y + m[1][2] * z,
                      m[2][0] * x + m[2][1] * y + m[2][2] * z)
    }
}

impl<'a, 'b> Mul<&'a Normal3f> for &'b Transform {
    type Output = Normal3f;

    fn mul(self, n: &'a Normal3f) -> Normal3f {
        let (x, y, z) = (n.x, n.y, n.z);
        let m = self.m_inv.m;

        Normal3f::new(m[0][0] * x + m[1][0] * y + m[2][0] * z,
                      m[0][1] * x + m[1][1] * y + m[2][1] * z,
                      m[0][2] * x + m[1][2] * y + m[2][2] * z)
    }
}

impl<'a, 'b> Mul<&'a Transform> for &'b Transform {
    type Output = Transform;

    fn mul(self, t: &'a Transform) -> Transform {
        Transform {
            m: &self.m * &t.m,
            m_inv: &t.m_inv * &self.m_inv,
        }
    }
}

impl Mul<Transform> for Transform {
    type Output = Transform;

    fn mul(self, t: Transform) -> Transform {
        Transform {
            m: &self.m * &t.m,
            m_inv: &t.m_inv * &self.m_inv,
        }
    }
}

impl<'a, 'b> Mul<&'a Bounds3f> for &'b Transform {
    type Output = Bounds3f;

    fn mul(self, b: &'a Bounds3f) -> Bounds3f {
        let mut ret = Bounds3f::from_point(&(self *
                                             &Point3f::new(b.p_min.x, b.p_min.y, b.p_min.z)));
        ret = Bounds3f::union_point(&ret,
                                    &(self * &Point3f::new(b.p_max.x, b.p_min.y, b.p_min.z)));
        ret = Bounds3f::union_point(&ret,
                                    &(self * &Point3f::new(b.p_min.x, b.p_max.y, b.p_min.z)));
        ret = Bounds3f::union_point(&ret,
                                    &(self * &Point3f::new(b.p_min.x, b.p_min.y, b.p_max.z)));
        ret = Bounds3f::union_point(&ret,
                                    &(self * &Point3f::new(b.p_min.x, b.p_max.y, b.p_max.z)));
        ret = Bounds3f::union_point(&ret,
                                    &(self * &Point3f::new(b.p_max.x, b.p_max.y, b.p_min.z)));
        ret = Bounds3f::union_point(&ret,
                                    &(self * &Point3f::new(b.p_max.x, b.p_min.y, b.p_max.z)));
        ret = Bounds3f::union_point(&ret,
                                    &(self * &Point3f::new(b.p_max.x, b.p_max.y, b.p_max.z)));

        ret
    }
}

#[allow(non_snake_case)]
pub fn solve_linear_system2x2(A: &[[f32; 2]; 2], B: &Vector2f) -> Option<(f32, f32)> {
    let det = A[0][0] * A[1][1] - A[0][1] * A[1][0];
    if det.abs() < 1e-10 {
        return None;
    }
    let x0 = (A[1][1] * B[0] - A[0][1] * B[1]) / det;
    let x1 = (A[0][0] * B[1] - A[1][0] * B[0]) / det;

    if x0.is_nan() || x1.is_nan() {
        return None;
    }
    Some((x0, x1))
}

#[test]
fn test_normal_transform() {
    let t = Transform::rotate(36.0, Vector3f::new(4.0, 5.0, 6.0));
    let t_inv = t.inverse();

    let v = Vector3f::x();
    let n = Vector3f::y();
    println!("v = {}, n = {}", v, n);
    assert_eq!(v.dot(&n), 0.0);

    let v2 = &t * &v;
    let n2 = t_inv.transform_normal(&Normal3f::from(n));
    println!("v = {}, n = {}", v2, n2);
    relative_eq!(v2.dotn(&n2), 0.0);
}
