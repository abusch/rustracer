use std::ops::{Sub, Mul, Add, Neg};
use point::Point;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vector {
    pub fn new(x: f32, y: f32, z: f32) -> Vector {
        Vector { x: x, y: y, z: z }
    }

    pub fn new1(v: f32) -> Vector {
        Vector { x: v, y: v, z: v }
    }

    pub fn zero() -> Vector {
        Vector { x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn length_2(&self) -> f32 {
        return self.x*self.x + self.y*self.y + self.z*self.z;
    }

    pub fn length(&self) -> f32 {
        return self.length_2().sqrt();
    }

    pub fn normalize(&self) -> Vector {
        let nor = self.length();
        let inv_nor = if nor == 0.0 { 0.0 } else { 1.0 / self.length() };
        return Vector::new(self.x * inv_nor, self.y * inv_nor, self.z * inv_nor);
    }

    pub fn dot(&self, rhs: &Vector) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn as_point(&self) -> Point {
        Point { x: self.x, y: self.y, z: self.z }
    }
}

impl From<Vector> for [u8; 3] {
    fn from(v: Vector) -> [u8; 3] {
        [
            (v.x * 255.0) as u8,
            (v.y * 255.0) as u8,
            (v.z * 255.0) as u8
        ]
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Vector {
        Vector::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Vector {
        Vector::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vector {
    type Output = Vector;
    fn mul(self, rhs: f32) -> Vector {
        Vector::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul<Vector> for Vector {
    type Output = Vector;
    fn mul(self, rhs: Vector) -> Vector {
        Vector::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Mul<Vector> for f32 {
    type Output = Vector;
    fn mul(self, rhs: Vector) -> Vector {
        Vector::new(self * rhs.x, self * rhs.y, self * rhs.z)
    }
}

impl Neg for Vector {
    type Output = Vector;
    fn neg(self) -> Vector {
        Vector::new(-self.x, -self.y, -self.z)
    }
}
