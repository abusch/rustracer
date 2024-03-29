use std::cmp::PartialOrd;
use std::f32;
use std::fmt;
use std::ops::{DivAssign, Index, SubAssign};

use num::{Bounded, Num, Signed};

use crate::geometry::{Point2, Point3, Vector2, Vector3};
use crate::ray::Ray;
use crate::{lerp, max, min, Point2f, Point2i, Point3f, Vector3f};

pub type Bounds3f = Bounds3<f32>;

/// Axis Aligned Bounding Box
#[derive(Debug, Copy, Clone)]
pub struct Bounds3<T: Num> {
    pub p_min: Point3<T>,
    pub p_max: Point3<T>,
}

impl<T> Bounds3<T>
where
    T: Bounded + PartialOrd + Into<f32> + Num + Signed + SubAssign + DivAssign + Copy,
{
    pub fn new() -> Bounds3<T> {
        let min = T::min_value();
        let max = T::max_value();
        Bounds3 {
            p_min: Point3::new(max, max, max),
            p_max: Point3::new(min, min, min),
        }
    }

    pub fn from_point(point: &Point3<T>) -> Bounds3<T> {
        Bounds3 {
            p_min: *point,
            p_max: *point,
        }
    }

    pub fn from_points(p1: &Point3<T>, p2: &Point3<T>) -> Bounds3<T> {
        Bounds3 {
            p_min: Point3::new(min(p1.x, p2.x), min(p1.y, p2.y), min(p1.z, p2.z)),
            p_max: Point3::new(max(p1.x, p2.x), max(p1.y, p2.y), max(p1.z, p2.z)),
        }
    }

    pub fn corner(&self, corner: usize) -> Point3<T> {
        Point3::new(
            self[corner & 1].x,
            self[if corner & 2 != 0 { 1 } else { 0 }].y,
            self[if corner & 4 != 0 { 1 } else { 0 }].z,
        )
    }

    pub fn extend(&mut self, p: Point3<T>) {
        if p.x < self.p_min.x {
            self.p_min.x = p.x
        }
        if p.y < self.p_min.y {
            self.p_min.y = p.y
        }
        if p.z < self.p_min.z {
            self.p_min.z = p.z
        }
        if p.x > self.p_max.x {
            self.p_max.x = p.x
        }
        if p.y > self.p_max.y {
            self.p_max.y = p.y
        }
        if p.z > self.p_max.z {
            self.p_max.z = p.z
        }
    }

    pub fn maximum_extent(&self) -> Axis {
        let v = self.p_max - self.p_min;
        if v.x > v.y {
            if v.x > v.z {
                Axis::X
            } else {
                Axis::Z
            }
        } else if v.y > v.z {
            Axis::Y
        } else {
            Axis::Z
        }
    }

    pub fn union(bbox1: &Bounds3<T>, bbox2: &Bounds3<T>) -> Bounds3<T> {
        let min = Point3::new(
            min(bbox1.p_min.x, bbox2.p_min.x),
            min(bbox1.p_min.y, bbox2.p_min.y),
            min(bbox1.p_min.z, bbox2.p_min.z),
        );
        let max = Point3::new(
            max(bbox1.p_max.x, bbox2.p_max.x),
            max(bbox1.p_max.y, bbox2.p_max.y),
            max(bbox1.p_max.z, bbox2.p_max.z),
        );

        Bounds3 {
            p_min: min,
            p_max: max,
        }
    }

    pub fn union_point(bbox: &Bounds3<T>, p: &Point3<T>) -> Bounds3<T> {
        let mut b = *bbox;
        b.extend(*p);
        b
    }

    pub fn intersect_p(&self, ray: &Ray) -> bool {
        let invdir = Vector3f::new(1.0 / ray.d.x, 1.0 / ray.d.y, 1.0 / ray.d.z);
        let sign = [
            (ray.d.x < 0.0) as usize,
            (ray.d.y < 0.0) as usize,
            (ray.d.z < 0.0) as usize,
        ];

        self.intersect_p_fast(ray, &invdir, &sign)
    }

    pub fn intersect_p_fast(&self, ray: &Ray, inv_dir: &Vector3f, dir_is_neg: &[usize; 3]) -> bool {
        // stats::inc_fast_bbox_isect();
        // Check intersection with X and Y slab
        let mut tmin = (self[dir_is_neg[0]].x.into() - ray.o.x) * inv_dir.x;
        let mut tmax = (self[1 - dir_is_neg[0]].x.into() - ray.o.x) * inv_dir.x;
        let tymin = (self[dir_is_neg[1]].y.into() - ray.o.y) * inv_dir.y;
        let tymax = (self[1 - dir_is_neg[1]].y.into() - ray.o.y) * inv_dir.y;
        if (tmin > tymax) || (tymin > tmax) {
            return false;
        }
        if tymin > tmin {
            tmin = tymin;
        }
        if tymax < tmax {
            tmax = tymax;
        }
        // Check intersection with Z slab
        let tzmin = (self[dir_is_neg[2]].z.into() - ray.o.z) * inv_dir.z;
        let tzmax = (self[1 - dir_is_neg[2]].z.into() - ray.o.z) * inv_dir.z;
        if (tmin > tzmax) || (tzmin > tmax) {
            return false;
        }
        if tzmin > tmin {
            tmin = tzmin;
        }
        if tzmax < tmax {
            tmax = tzmax;
        }

        tmin < ray.t_max && tmax > 0.0
    }

    /// Linearly interpolate a point inside the bounds
    pub fn lerp(&self, t: &Point3<T>) -> Point3<T> {
        Point3::new(
            lerp(t.x, self.p_min.x, self.p_max.x),
            lerp(t.y, self.p_min.y, self.p_max.y),
            lerp(t.z, self.p_min.z, self.p_max.z),
        )
    }

    pub fn inside(&self, p: &Point3<T>) -> bool {
        p.x >= self.p_min.x
            && p.x <= self.p_max.x
            && p.y >= self.p_min.y
            && p.y <= self.p_max.y
            && p.z >= self.p_min.z
            && p.z <= self.p_max.z
    }

    pub fn offset(&self, p: &Point3<T>) -> Vector3<T> {
        let mut o = *p - self.p_min;
        if self.p_max.x > self.p_min.x {
            o.x /= self.p_max.x - self.p_min.x;
        }
        if self.p_max.y > self.p_min.y {
            o.y /= self.p_max.y - self.p_min.y;
        }
        if self.p_max.z > self.p_min.z {
            o.z /= self.p_max.z - self.p_min.z;
        }

        o
    }

    pub fn diagonal(&self) -> Vector3<T> {
        self.p_max - self.p_min
    }
}

impl Bounds3<f32> {
    /// Compute the bounding sphere of the current bounding box, and returns its center and radius.
    pub fn bounding_sphere(&self) -> (Point3f, f32) {
        let center = Point3f::new(
            (self.p_min.x + self.p_max.x) / 2.0,
            (self.p_min.y + self.p_max.y) / 2.0,
            (self.p_min.z + self.p_max.z) / 2.0,
        );
        let radius = if self.inside(&center) {
            (self.p_max - center).length()
        } else {
            0.0
        };

        (center, radius)
    }

    pub fn surface_area(&self) -> f32 {
        let d = self.diagonal();
        2.0 * (d.x * d.y + d.x * d.z + d.y * d.z)
    }
}

impl<T> Default for Bounds3<T>
where
    T: Bounded + PartialOrd + Into<f32> + Num + Signed + Copy + SubAssign + DivAssign,
{
    fn default() -> Self {
        Self::new()
    }
}

pub type Bounds2i = Bounds2<i32>;
pub type Bounds2f = Bounds2<f32>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Bounds2<T: Num> {
    pub p_min: Point2<T>,
    pub p_max: Point2<T>,
}

impl<T> Bounds2<T>
where
    T: Bounded + PartialOrd + fmt::Display + Num + Signed + Copy,
{
    pub fn new() -> Bounds2<T> {
        let min = T::min_value();
        let max = T::max_value();
        Bounds2 {
            p_min: Point2::new(max, max),
            p_max: Point2::new(min, min),
        }
    }

    pub fn from_elements(x1: T, y1: T, x2: T, y2: T) -> Bounds2<T> {
        Bounds2 {
            p_min: Point2::new(x1, y1),
            p_max: Point2::new(x2, y2),
        }
    }

    pub fn from_point(point: &Point2<T>) -> Bounds2<T> {
        Bounds2 {
            p_min: *point,
            p_max: *point,
        }
    }

    pub fn from_points(p1: &Point2<T>, p2: &Point2<T>) -> Bounds2<T> {
        Bounds2 {
            p_min: Point2::new(min(p1.x, p2.x), min(p1.y, p2.y)),
            p_max: Point2::new(max(p1.x, p2.x), max(p1.y, p2.y)),
        }
    }

    pub fn extend(&mut self, p: Point2<T>) {
        if p.x < self.p_min.x {
            self.p_min.x = p.x
        }
        if p.y < self.p_min.y {
            self.p_min.y = p.y
        }
        if p.x > self.p_max.x {
            self.p_max.x = p.x
        }
        if p.y > self.p_max.y {
            self.p_max.y = p.y
        }
    }

    // pub fn maximum_extent(&self) -> Axis {
    //     let v = self.p_max - self.p_min;
    //     if v.x > v.y {
    //         if v.x > v.z { Axis::X } else { Axis::Z }
    //     } else if v.y > v.z {
    //         Axis::Y
    //     } else {
    //         Axis::Z
    //     }
    // }

    pub fn union(bbox1: &Bounds2<T>, bbox2: &Bounds2<T>) -> Bounds2<T> {
        let min = Point2::new(
            min(bbox1.p_min.x, bbox2.p_min.x),
            min(bbox1.p_min.y, bbox2.p_min.y),
        );
        let max = Point2::new(
            max(bbox1.p_max.x, bbox2.p_max.x),
            max(bbox1.p_max.y, bbox2.p_max.y),
        );

        Bounds2 {
            p_min: min,
            p_max: max,
        }
    }

    pub fn union_point(bbox: &Bounds2<T>, p: &Point2<T>) -> Bounds2<T> {
        let mut b = *bbox;
        b.extend(*p);
        b
    }

    /// Linearly interpolate a point inside the bounds
    pub fn lerp(&self, t: &Point2<T>) -> Point2<T> {
        Point2::new(
            lerp(t.x, self.p_min.x, self.p_max.x),
            lerp(t.y, self.p_min.y, self.p_max.y),
        )
    }

    pub fn inside(&self, p: &Point2<T>) -> bool {
        p.x >= self.p_min.x && p.x <= self.p_max.x && p.y >= self.p_min.y && p.y <= self.p_max.y
    }

    pub fn inside_exclusive(&self, p: &Point2<T>) -> bool {
        p.x >= self.p_min.x && p.x < self.p_max.x && p.y >= self.p_min.y && p.y < self.p_max.y
    }

    pub fn area(&self) -> T {
        (self.p_max.x - self.p_min.x) * (self.p_max.y - self.p_min.y)
    }

    pub fn intersect(bbox1: &Bounds2<T>, bbox2: &Bounds2<T>) -> Bounds2<T> {
        let p_min = Point2::new(
            max(bbox1.p_min.x, bbox2.p_min.x),
            max(bbox1.p_min.y, bbox2.p_min.y),
        );
        let p_max = Point2::new(
            min(bbox1.p_max.x, bbox2.p_max.x),
            min(bbox1.p_max.y, bbox2.p_max.y),
        );

        Bounds2 { p_min, p_max }
    }

    pub fn diagonal(&self) -> Vector2<T> {
        self.p_max - self.p_min
    }
}

impl<T> Default for Bounds2<T>
where
    T: Bounded + PartialOrd + fmt::Display + Num + Signed + Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<usize> for Bounds3<T>
where
    T: Copy + Num,
{
    type Output = Point3<T>;

    fn index(&self, i: usize) -> &Point3<T> {
        match i {
            0 => &self.p_min,
            1 => &self.p_max,
            _ => panic!("Invalid index!"),
        }
    }
}

pub struct Bounds2Iterator<'a> {
    p: Point2i,
    bounds: &'a Bounds2i,
}

impl<'a> Iterator for Bounds2Iterator<'a> {
    type Item = Point2i;

    fn next(&mut self) -> Option<Point2i> {
        if self.bounds.p_max.x <= self.bounds.p_min.x || self.bounds.p_max.y <= self.bounds.p_min.y
        {
            // Handle degenerate bounds explicitly
            return None;
        }
        self.p.x += 1;
        if self.p.x == self.bounds.p_max.x {
            self.p.x = self.bounds.p_min.x;
            self.p.y += 1;
        }
        if self.p.y >= self.bounds.p_max.y {
            None
        } else {
            Some(self.p)
        }
    }
}

impl<'a> IntoIterator for &'a Bounds2<i32> {
    type Item = Point2i;
    type IntoIter = Bounds2Iterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Bounds2Iterator {
            // Need to start 1 before p_min.x as next() will be called to get the first element
            p: Point2i::new(self.p_min.x - 1, self.p_min.y),
            bounds: self,
        }
    }
}

impl From<Bounds2i> for Bounds2f {
    fn from(b: Bounds2i) -> Self {
        Bounds2f::from_points(
            &Point2f::new(b.p_min.x as f32, b.p_min.y as f32),
            &Point2f::new(b.p_max.x as f32, b.p_max.y as f32),
        )
    }
}

impl From<Bounds2f> for Bounds2i {
    fn from(b: Bounds2f) -> Self {
        Bounds2i::from_points(
            &Point2i::new(b.p_min.x as i32, b.p_min.y as i32),
            &Point2i::new(b.p_max.x as i32, b.p_max.y as i32),
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Index<Axis> for Point3<f32> {
    type Output = f32;

    fn index(&self, axis: Axis) -> &f32 {
        match axis {
            Axis::X => &self.x,
            Axis::Y => &self.y,
            Axis::Z => &self.z,
        }
    }
}

impl Index<Axis> for Vector3<f32> {
    type Output = f32;

    fn index(&self, axis: Axis) -> &f32 {
        match axis {
            Axis::X => &self.x,
            Axis::Y => &self.y,
            Axis::Z => &self.z,
        }
    }
}

impl<T> fmt::Display for Bounds2<T>
where
    T: fmt::Display + Num,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} → {}", self.p_min, self.p_max)
    }
}

impl<T> fmt::Display for Bounds3<T>
where
    T: fmt::Display + Num,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} → {}", self.p_min, self.p_max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds2_iterator_basics() {
        let b = Bounds2i::from_elements(0, 1, 2, 3);
        let e = [
            Point2i::new(0, 1),
            Point2i::new(1, 1),
            Point2i::new(0, 2),
            Point2i::new(1, 2),
        ];
        for (offset, p) in b.into_iter().enumerate() {
            assert!(offset < e.len());
            assert_eq!(e[offset], p);
        }
    }

    #[test]
    fn bounds2_iterator_degenerate() {
        {
            let b = Bounds2i::from_elements(0, 0, 0, 10);
            for p in &b {
                panic!("should not have reached this point! p = {}", p);
            }
        }

        {
            let b2 = Bounds2i::from_elements(0, 0, 4, 0);
            for p in &b2 {
                panic!("should not have reached this point! p = {}", p);
            }
        }

        {
            let b3 = Bounds2i::new();
            for p in &b3 {
                panic!("should not have reached this point! p = {}", p);
            }
        }
    }

    #[test]
    fn test_bounds2_iterator() {
        let bounds = Bounds2i::from_points(&Point2i::new(0, 1), &Point2i::new(2, 3));

        let points: Vec<Point2i> = bounds.into_iter().collect();

        assert_eq!(points.len(), 4);
        assert_eq!(points[0], Point2i::new(0, 1));
        assert_eq!(points[1], Point2i::new(1, 1));
        assert_eq!(points[2], Point2i::new(0, 2));
        assert_eq!(points[3], Point2i::new(1, 2));
    }

    #[test]
    fn test_bounds2_union() {
        let a = Bounds2i::from_elements(-10, -10, 0, 20);
        let b = Bounds2i::new();
        let c = Bounds2i::union(&a, &b);

        assert_eq!(a, c);
        assert_eq!(b, Bounds2i::union(&b, &b));

        let d = Bounds2i::from_point(&Point2i::new(-15, 10));
        let e = Bounds2i::union(&a, &d);
        assert_eq!(Bounds2i::from_elements(-15, -10, 0, 20), e);
    }
}
