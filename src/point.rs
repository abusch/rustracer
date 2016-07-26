use std::ops::{Sub, Add, Index, IndexMut};

use vector::Vector;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Point {
    pub fn new(x: f32, y: f32, z: f32) -> Point {
        Point { x: x, y: y, z: z }
    }

    pub fn new1(v: f32) -> Point {
        Point { x: v, y: v, z: v }
    }

    pub fn origin() -> Point {
        Point { x: 0.0, y: 0.0, z: 0.0 }
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        Point::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Add<Vector> for Point {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Vector {
        Vector::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Point {
    type Output = Vector;

    fn sub(self, rhs: Point) -> Vector {
        Vector::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Sub<Vector> for Point {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Vector {
        Vector::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

// impl Mul<f32> for Point {
//     type Output = Point;
//     fn mul(self, rhs: f32) -> Point {
//         Point::new(self.x * rhs, self.y * rhs, self.z * rhs)
//     }
// }

// impl Mul<Point> for Point {
//     type Output = Point;
//     fn mul(self, rhs: Point) -> Point {
//         Point::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
//     }
// }

// impl Mul<Point> for f32 {
//     type Output = Point;
//     fn mul(self, rhs: Point) -> Point {
//         Point::new(self * rhs.x, self * rhs.y, self * rhs.z)
//     }
// }

// impl Neg for Point {
//     type Output = Point;
//     fn neg(self) -> Point {
//         Point::new(-self.x, -self.y, -self.z)
//     }
// }

impl Index<usize> for Point {
    type Output = f32;
    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Invalid index into point"),
        }
    }
}

impl IndexMut<usize> for Point {
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Invalid index into point"),
        }
    }
}
