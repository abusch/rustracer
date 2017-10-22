use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub,
               SubAssign};
use std::convert::From;
use std::fmt::{Display, Error, Formatter};

use {Normal3, Point3};
use num::{Num, Zero};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector2<T> {
    pub fn new(x: T, y: T) -> Vector2<T> {
        Vector2 { x, y }
    }
}

impl Vector2<f32> {
    pub fn has_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(&self) -> f32 {
        f32::sqrt(self.length_squared())
    }
}

// Operators
impl<T> Add<Vector2<T>> for Vector2<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Vector2<T>;

    fn add(self, rhs: Vector2<T>) -> Vector2<T> {
        Vector2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> AddAssign<Vector2<T>> for Vector2<T>
where
    T: AddAssign + Copy,
{
    fn add_assign(&mut self, other: Vector2<T>) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl<T> Sub<Vector2<T>> for Vector2<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Vector2<T>;

    fn sub(self, rhs: Vector2<T>) -> Vector2<T> {
        Vector2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> SubAssign<Vector2<T>> for Vector2<T>
where
    T: SubAssign + Copy,
{
    fn sub_assign(&mut self, other: Vector2<T>) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl<T> Div<T> for Vector2<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Vector2<T>;

    fn div(self, v: T) -> Vector2<T> {
        Vector2 {
            x: self.x / v,
            y: self.y / v,
        }
    }
}

impl<T> DivAssign<T> for Vector2<T>
where
    T: DivAssign + Copy,
{
    fn div_assign(&mut self, v: T) {
        self.x /= v;
        self.y /= v;
    }
}

impl<T> Mul<T> for Vector2<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vector2<T>;

    fn mul(self, v: T) -> Vector2<T> {
        Vector2 {
            x: self.x * v,
            y: self.y * v,
        }
    }
}

impl<T> MulAssign<T> for Vector2<T>
where
    T: MulAssign + Copy,
{
    fn mul_assign(&mut self, v: T) {
        self.x *= v;
        self.y *= v;
    }
}

impl<T> Neg for Vector2<T>
where
    T: Neg<Output = T>,
{
    type Output = Vector2<T>;

    fn neg(self) -> Vector2<T> {
        Vector2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> Index<usize> for Vector2<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        match i {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Invalid index into vector"),
        }
    }
}

impl<T> IndexMut<usize> for Vector2<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Invalid index into vector"),
        }
    }
}

impl<T> Default for Vector2<T>
where
    T: Default,
{
    fn default() -> Self {
        Vector2 {
            x: T::default(),
            y: T::default(),
        }
    }
}

impl<T> Zero for Vector2<T>
where
    T: Num + Copy,
{
    fn zero() -> Vector2<T> {
        Vector2::new(T::zero(), T::zero())
    }

    fn is_zero(&self) -> bool {
        self.x == T::zero() && self.y == T::zero()
    }
}

impl<T> Display for Vector2<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vector3<T>
where
    T: Num + Copy,
{
    pub fn new(x: T, y: T, z: T) -> Vector3<T> {
        Vector3 { x, y, z }
    }

    pub fn dot(&self, v: &Vector3<T>) -> T {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    pub fn dotn(&self, v: &Normal3<T>) -> T {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    pub fn x() -> Vector3<T> {
        Vector3::new(T::one(), T::zero(), T::zero())
    }

    pub fn y() -> Vector3<T> {
        Vector3::new(T::zero(), T::one(), T::zero())
    }

    pub fn z() -> Vector3<T> {
        Vector3::new(T::zero(), T::zero(), T::one())
    }
}

impl Vector3<f32> {
    pub fn has_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan() || self.z.is_nan()
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f32 {
        f32::sqrt(self.length_squared())
    }

    pub fn normalize(&self) -> Vector3<f32> {
        *self / self.length()
    }

    pub fn cross(&self, v: &Vector3<f32>) -> Vector3<f32> {
        Vector3::new(
            (self.y * v.z) - (self.z * v.y),
            (self.z * v.x) - (self.x * v.z),
            (self.x * v.y) - (self.y * v.x),
        )
    }

    pub fn abs(&self) -> Vector3<f32> {
        Vector3::new(f32::abs(self.x), f32::abs(self.y), f32::abs(self.z))
    }

    // TODO cross_normal()
}

// Operators
impl<T> Add<Vector3<T>> for Vector3<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Vector3<T>;

    fn add(self, rhs: Vector3<T>) -> Vector3<T> {
        Vector3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T> AddAssign<Vector3<T>> for Vector3<T>
where
    T: AddAssign + Copy,
{
    fn add_assign(&mut self, other: Vector3<T>) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl<T> Sub<Vector3<T>> for Vector3<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Vector3<T>;

    fn sub(self, rhs: Vector3<T>) -> Vector3<T> {
        Vector3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T> SubAssign<Vector3<T>> for Vector3<T>
where
    T: SubAssign + Copy,
{
    fn sub_assign(&mut self, other: Vector3<T>) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl<T> Div<T> for Vector3<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Vector3<T>;

    fn div(self, v: T) -> Vector3<T> {
        Vector3 {
            x: self.x / v,
            y: self.y / v,
            z: self.z / v,
        }
    }
}

impl<T> DivAssign<T> for Vector3<T>
where
    T: DivAssign + Copy,
{
    fn div_assign(&mut self, v: T) {
        self.x /= v;
        self.y /= v;
        self.z /= v;
    }
}

impl<T> Mul<T> for Vector3<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vector3<T>;

    fn mul(self, v: T) -> Vector3<T> {
        Vector3 {
            x: self.x * v,
            y: self.y * v,
            z: self.z * v,
        }
    }
}

impl Mul<Vector3<f32>> for f32 {
    type Output = Vector3<f32>;

    fn mul(self, v: Vector3<f32>) -> Vector3<f32> {
        Vector3 {
            x: self * v.x,
            y: self * v.y,
            z: self * v.z,
        }
    }
}

impl<T> MulAssign<T> for Vector3<T>
where
    T: MulAssign + Copy,
{
    fn mul_assign(&mut self, v: T) {
        self.x *= v;
        self.y *= v;
        self.z *= v;
    }
}

impl<T> Neg for Vector3<T>
where
    T: Neg<Output = T>,
{
    type Output = Vector3<T>;

    fn neg(self) -> Vector3<T> {
        Vector3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl<T> Index<usize> for Vector3<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Invalid index into vector"),
        }
    }
}

impl<T> IndexMut<usize> for Vector3<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Invalid index into vector"),
        }
    }
}

impl<T> From<Point3<T>> for Vector3<T>
where
    T: Num + Copy,
{
    fn from(p: Point3<T>) -> Vector3<T> {
        Vector3::new(p.x, p.y, p.z)
    }
}

impl<T> From<Normal3<T>> for Vector3<T>
where
    T: Num + Copy,
{
    fn from(n: Normal3<T>) -> Vector3<T> {
        Vector3::new(n.x, n.y, n.z)
    }
}

impl<T> Default for Vector3<T>
where
    T: Default,
{
    fn default() -> Self {
        Vector3 {
            x: T::default(),
            y: T::default(),
            z: T::default(),
        }
    }
}

impl<T> Zero for Vector3<T>
where
    T: Num + Copy,
{
    fn zero() -> Vector3<T> {
        Vector3::new(T::zero(), T::zero(), T::zero())
    }

    fn is_zero(&self) -> bool {
        self.x == T::zero() && self.y == T::zero() && self.z == T::zero()
    }
}

impl<T> Display for Vector3<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add() {
        let v1 = Vector2::new(1.0, 2.0);
        let v2 = Vector2::new(3.0, 4.0);
        assert_eq!(v1 + v2, Vector2::new(4.0, 6.0));
    }
}
