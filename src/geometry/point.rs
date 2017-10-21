use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub,
               SubAssign};
use std::fmt::{Display, Error, Formatter};

use num::{abs, Num, Signed, Zero};

use geometry::{Vector2, Vector3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point2<T>
where
    T: Num + Signed + Copy,
{
    pub fn new(x: T, y: T) -> Point2<T> {
        Point2 { x, y }
    }

    pub fn abs(&self) -> Point2<T> {
        Point2::new(abs(self.x), abs(self.y))
    }
}

impl Point2<f32> {
    pub fn has_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }
}

// Operators
// Point2 + Point2 -> Point2
impl<T> Add<Point2<T>> for Point2<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Point2<T>;

    fn add(self, rhs: Point2<T>) -> Point2<T> {
        Point2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

// Point2 + Vector2 -> Point2
impl<T> Add<Vector2<T>> for Point2<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Point2<T>;

    fn add(self, rhs: Vector2<T>) -> Point2<T> {
        Point2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

// Point2 += Point2 -> Point2
impl<T> AddAssign<Point2<T>> for Point2<T>
where
    T: AddAssign + Copy,
{
    fn add_assign(&mut self, other: Point2<T>) {
        self.x += other.x;
        self.y += other.y;
    }
}

// Point2 += Vector2 -> Point2
impl<T> AddAssign<Vector2<T>> for Point2<T>
where
    T: AddAssign + Copy,
{
    fn add_assign(&mut self, other: Vector2<T>) {
        self.x += other.x;
        self.y += other.y;
    }
}

// Point2 - Point2 -> Vector2
impl<T> Sub<Point2<T>> for Point2<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Vector2<T>;

    fn sub(self, rhs: Point2<T>) -> Vector2<T> {
        Vector2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

// Point2 - Vector2 -> Point2
impl<T> Sub<Vector2<T>> for Point2<T>
where
    T: Num + Signed + Copy,
{
    type Output = Point2<T>;

    fn sub(self, rhs: Vector2<T>) -> Point2<T> {
        Point2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

// Point2 -= Vector2 -> Point2
impl<T> SubAssign<Vector2<T>> for Point2<T>
where
    T: SubAssign + Copy,
{
    fn sub_assign(&mut self, other: Vector2<T>) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl<T> Div<T> for Point2<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Point2<T>;

    fn div(self, v: T) -> Point2<T> {
        Point2 {
            x: self.x / v,
            y: self.y / v,
        }
    }
}

impl<T> DivAssign<T> for Point2<T>
where
    T: DivAssign + Copy,
{
    fn div_assign(&mut self, v: T) {
        self.x /= v;
        self.y /= v;
    }
}

impl<T> Mul<T> for Point2<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Point2<T>;

    fn mul(self, v: T) -> Point2<T> {
        Point2 {
            x: self.x * v,
            y: self.y * v,
        }
    }
}

impl Mul<Point2<f32>> for f32 {
    type Output = Point2<f32>;

    fn mul(self, p: Point2<f32>) -> Point2<f32> {
        Point2::new(self * p.x, self * p.y)
    }
}

impl<T> MulAssign<T> for Point2<T>
where
    T: MulAssign + Copy,
{
    fn mul_assign(&mut self, v: T) {
        self.x *= v;
        self.y *= v;
    }
}

impl<T> Neg for Point2<T>
where
    T: Neg<Output = T>,
{
    type Output = Point2<T>;

    fn neg(self) -> Point2<T> {
        Point2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> Index<usize> for Point2<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        match i {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Invalid index into point"),
        }
    }
}

impl<T> IndexMut<usize> for Point2<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Invalid index into point"),
        }
    }
}

impl<T> Default for Point2<T>
where
    T: Default,
{
    fn default() -> Self {
        Point2 {
            x: T::default(),
            y: T::default(),
        }
    }
}

impl<T> Display for Point2<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl<T> Zero for Point2<T>
where
    T: Num + Signed + Copy,
{
    fn zero() -> Point2<T> {
        Point2::new(T::zero(), T::zero())
    }

    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Point3<T>
where
    T: Num + Signed + Copy,
{
    pub fn new(x: T, y: T, z: T) -> Point3<T> {
        Point3 { x, y, z }
    }

    pub fn abs(&self) -> Point3<T> {
        Point3::new(abs(self.x), abs(self.y), abs(self.z))
    }
}

impl Point3<f32> {
    pub fn has_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan() || self.z.is_nan()
    }
}

// Operators
// Point3 + Point3 -> Point3
impl<T> Add<Point3<T>> for Point3<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Point3<T>;

    fn add(self, rhs: Point3<T>) -> Point3<T> {
        Point3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

// Point3 + Vector3 -> Point3
impl<T> Add<Vector3<T>> for Point3<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Point3<T>;

    fn add(self, rhs: Vector3<T>) -> Point3<T> {
        Point3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

// Point3 += Vector3 -> Point3
impl<T> AddAssign<Vector3<T>> for Point3<T>
where
    T: AddAssign + Copy,
{
    fn add_assign(&mut self, other: Vector3<T>) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

// Point3 += Point3 -> Point3
impl<T> AddAssign<Point3<T>> for Point3<T>
where
    T: AddAssign + Copy,
{
    fn add_assign(&mut self, other: Point3<T>) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

// Point3 - Point3 -> Vector3
impl<T> Sub<Point3<T>> for Point3<T>
where
    T: Sub<Output = T> + Copy + Num,
{
    type Output = Vector3<T>;

    fn sub(self, rhs: Point3<T>) -> Vector3<T> {
        Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

// Point3 - Vector3 -> Point3
impl<T> Sub<Vector3<T>> for Point3<T>
where
    T: Num + Signed + Copy,
{
    type Output = Point3<T>;

    fn sub(self, rhs: Vector3<T>) -> Point3<T> {
        Point3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

// Point3 -= Vector3 -> Point3
impl<T> SubAssign<Vector3<T>> for Point3<T>
where
    T: SubAssign + Copy,
{
    fn sub_assign(&mut self, other: Vector3<T>) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl<T> Div<T> for Point3<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Point3<T>;

    fn div(self, v: T) -> Point3<T> {
        Point3 {
            x: self.x / v,
            y: self.y / v,
            z: self.z / v,
        }
    }
}

impl<T> DivAssign<T> for Point3<T>
where
    T: DivAssign + Copy,
{
    fn div_assign(&mut self, v: T) {
        self.x /= v;
        self.y /= v;
        self.z /= v;
    }
}

impl<T> Mul<T> for Point3<T>
where
    T: Num + Signed + Copy,
{
    type Output = Point3<T>;

    fn mul(self, v: T) -> Point3<T> {
        Point3::new(self.x * v, self.y * v, self.z * v)
    }
}

impl Mul<Point3<f32>> for f32 {
    type Output = Point3<f32>;

    fn mul(self, p: Point3<f32>) -> Point3<f32> {
        Point3::new(self * p.x, self * p.y, self * p.z)
    }
}

impl<T> MulAssign<T> for Point3<T>
where
    T: MulAssign + Copy,
{
    fn mul_assign(&mut self, v: T) {
        self.x *= v;
        self.y *= v;
        self.z *= v;
    }
}

impl<T> Neg for Point3<T>
where
    T: Neg<Output = T>,
{
    type Output = Point3<T>;

    fn neg(self) -> Point3<T> {
        Point3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl<T> Index<usize> for Point3<T> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Invalid index into point"),
        }
    }
}

impl<T> IndexMut<usize> for Point3<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Invalid index into point"),
        }
    }
}

impl<T> From<Vector3<T>> for Point3<T>
where
    T: Num + Signed + Copy,
{
    fn from(p: Vector3<T>) -> Point3<T> {
        Point3::new(p.x, p.y, p.z)
    }
}

impl<T> Default for Point3<T>
where
    T: Default,
{
    fn default() -> Self {
        Point3 {
            x: T::default(),
            y: T::default(),
            z: T::default(),
        }
    }
}

impl<T> Display for Point3<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

impl<T> Zero for Point3<T>
where
    T: Num + Signed + Copy,
{
    fn zero() -> Point3<T> {
        Point3::new(T::zero(), T::zero(), T::zero())
    }

    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero() && self.z.is_zero()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add() {
        let v1 = Point2::new(1.0, 2.0);
        let v2 = Point2::new(3.0, 4.0);
        assert_eq!(v1 + v2, Point2::new(4.0, 6.0));
    }
}
