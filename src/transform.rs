use alga::linear::Transformation;
use na::{self, Matrix3, Matrix4, Vector3, Vector4, U3};

use {Vector3f, Point3f, Transform, gamma};

/// Transform the given point using the given transformation and also return a vector of the
/// absolute error introduced for each coordinate.
pub fn transform_point(t: &Transform, p: &Point3f) -> (Point3f, Vector3f) {
    let (x, y, z) = (p.x, p.y, p.z);
    let tp = *t * *p;
    let m: Matrix4<f32> = t.to_homogeneous();
    let x_abs_sum = (m[(0, 0)] * x).abs() + (m[(0, 1)] * y).abs() + (m[(0, 2)] * z).abs() +
                    m[(0, 3)].abs();
    let y_abs_sum = (m[(1, 0)] * x).abs() + (m[(1, 1)] * y).abs() + (m[(1, 2)] * z).abs() +
                    m[(1, 3)].abs();
    let z_abs_sum = (m[(2, 0)] * x).abs() + (m[(2, 1)] * y).abs() + (m[(2, 2)] * z).abs() +
                    m[(2, 3)].abs();
    let p_err = gamma(3) * Vector3f::new(x_abs_sum, y_abs_sum, z_abs_sum);

    (tp, p_err)
}

pub fn transform_point_with_error(t: &Transform,
                                  p: &Point3f,
                                  p_error: &Vector3f)
                                  -> (Point3f, Vector3f) {
    let (x, y, z) = (p.x, p.y, p.z);
    let tp = *t * *p;
    let m: Matrix4<f32> = t.to_homogeneous();
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
pub fn transform_vector(t: &Transform, v: &Vector3f) -> (Vector3f, Vector3f) {
    let (x, y, z) = (v.x, v.y, v.z);
    let tv = *t * *v;
    let m: Matrix4<f32> = t.to_homogeneous();
    let x_abs_sum = na::abs(&(m[(0, 0)] * x)) + na::abs(&(m[(0, 1)] * y)) +
                    na::abs(&(m[(0, 2)] * z)) + na::abs(&m[(0, 3)]);
    let y_abs_sum = na::abs(&(m[(1, 0)] * x)) + na::abs(&(m[(1, 1)] * y)) +
                    na::abs(&(m[(1, 2)] * z)) + na::abs(&m[(1, 3)]);
    let z_abs_sum = na::abs(&(m[(2, 0)] * x)) + na::abs(&(m[(2, 1)] * y)) +
                    na::abs(&(m[(2, 2)] * z)) + na::abs(&m[(2, 3)]);
    let v_err = gamma(3) * Vector3f::new(x_abs_sum, y_abs_sum, z_abs_sum);

    (tv, v_err)
}

pub fn transform_normal(normal: &Vector3f, transform: &Transform) -> Vector3f {
    let m = transform.inverse().to_homogeneous().transpose();
    // Vector3::from_homogeneous(n_hom).unwrap_or_else(|| Vector3::new(n_hom.x, n_hom.y, n_hom.z))
    m.fixed_slice::<U3, U3>(0, 0) * *normal
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

#[cfg(test)]
use na::{Dot, Inverse};

#[test]
fn test_normal_transform() {
    let t = Transform::new(Vector3f::new(0.0, 0.0, 0.0),
                           Vector3f::new(4.0, 5.0, 6.0),
                           4.0);
    let t_inv = t.inverse().unwrap();

    let v = Vector3f::x();
    let n = Vector3f::y();
    println!("v = {}, n = {}", v, n);
    assert_eq!(v.dot(&n), 0.0);

    let v2 = t * v;
    let n2 = transform_normal(&n, &t_inv);
    println!("v = {}, n = {}", v2, n2);
    relative_eq!(v2.dot(&n2), 0.0);
}
