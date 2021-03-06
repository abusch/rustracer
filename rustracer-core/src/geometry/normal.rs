use std::convert::From;
use std::fmt::{Display, Error, Formatter};
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

use num::{Num, Zero};

use crate::geometry::Vector3;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Normal3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Normal3<T>
where
    T: Num + Copy,
{
    pub fn new(x: T, y: T, z: T) -> Normal3<T> {
        Normal3 { x, y, z }
    }

    pub fn dot(&self, v: &Vector3<T>) -> T {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    pub fn dotn(&self, v: &Normal3<T>) -> T {
        self.x * v.x + self.y * v.y + self.z * v.z
    }
}

impl Normal3<f32> {
    pub fn has_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan() || self.z.is_nan()
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f32 {
        f32::sqrt(self.length_squared())
    }

    pub fn normalize(&self) -> Normal3<f32> {
        *self / self.length()
    }

    pub fn abs(&self) -> Normal3<f32> {
        Normal3::new(self.x.abs(), self.y.abs(), self.z.abs())
    }
}

// Operators
impl<T> Add<Normal3<T>> for Normal3<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Normal3<T>;

    fn add(self, rhs: Normal3<T>) -> Normal3<T> {
        Normal3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T> AddAssign<Normal3<T>> for Normal3<T>
where
    T: AddAssign + Copy,
{
    fn add_assign(&mut self, other: Normal3<T>) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl<T> Sub<Normal3<T>> for Normal3<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Normal3<T>;

    fn sub(self, rhs: Normal3<T>) -> Normal3<T> {
        Normal3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T> SubAssign<Normal3<T>> for Normal3<T>
where
    T: SubAssign + Copy,
{
    fn sub_assign(&mut self, other: Normal3<T>) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl<T> Div<T> for Normal3<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Normal3<T>;

    fn div(self, v: T) -> Normal3<T> {
        Normal3 {
            x: self.x / v,
            y: self.y / v,
            z: self.z / v,
        }
    }
}

impl<T> DivAssign<T> for Normal3<T>
where
    T: DivAssign + Copy,
{
    fn div_assign(&mut self, v: T) {
        self.x /= v;
        self.y /= v;
        self.z /= v;
    }
}

impl<T> Mul<T> for Normal3<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Normal3<T>;

    fn mul(self, v: T) -> Normal3<T> {
        Normal3 {
            x: self.x * v,
            y: self.y * v,
            z: self.z * v,
        }
    }
}

impl Mul<Normal3<f32>> for f32 {
    type Output = Normal3<f32>;

    fn mul(self, v: Normal3<f32>) -> Normal3<f32> {
        Normal3 {
            x: self * v.x,
            y: self * v.y,
            z: self * v.z,
        }
    }
}

impl<T> MulAssign<T> for Normal3<T>
where
    T: MulAssign + Copy,
{
    fn mul_assign(&mut self, v: T) {
        self.x *= v;
        self.y *= v;
        self.z *= v;
    }
}

impl<T> Neg for Normal3<T>
where
    T: Neg<Output = T>,
{
    type Output = Normal3<T>;

    fn neg(self) -> Normal3<T> {
        Normal3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl<T> Index<usize> for Normal3<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Invalid index into normal"),
        }
    }
}

impl<T> IndexMut<usize> for Normal3<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Invalid index into normal"),
        }
    }
}

impl<T> Default for Normal3<T>
where
    T: Default,
{
    fn default() -> Self {
        Normal3 {
            x: T::default(),
            y: T::default(),
            z: T::default(),
        }
    }
}

impl<T> Zero for Normal3<T>
where
    T: Num + Copy,
{
    fn zero() -> Normal3<T> {
        Normal3::new(T::zero(), T::zero(), T::zero())
    }

    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero() && self.z.is_zero()
    }
}

impl<T> From<Vector3<T>> for Normal3<T>
where
    T: Num + Copy,
{
    fn from(v: Vector3<T>) -> Normal3<T> {
        Normal3::new(v.x, v.y, v.z)
    }
}

impl<T> Display for Normal3<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}
