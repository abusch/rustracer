use std::f32;
use std::ops::Index;
use std::cmp::PartialOrd;
use {Vector, Point, lerp};
use na::{Point3, Norm};
use ray::Ray;
use stats;
use num::Bounded;
use na::BaseNum;

pub type Bounds3f = Bounds3<f32>;

/// Axis Aligned Bounding Box
#[derive(Debug, Copy, Clone)]
pub struct Bounds3<T> {
    pub p_min: Point3<T>,
    pub p_max: Point3<T>,
}

impl<T> Bounds3<T>
    where T: BaseNum + Bounded + PartialOrd + Into<f32>
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

    pub fn from_points(min: &Point3<T>, max: &Point3<T>) -> Bounds3<T> {
        assert!(min.x <= max.x && min.y <= max.y && min.z <= max.z,
                "Invalid bounds");
        Bounds3 {
            p_min: *min,
            p_max: *max,
        }
    }

    pub fn corner(&self, corner: usize) -> Point3<T> {
        Point3::new(self[corner & 1].x,
                    self[if corner & 2 != 0 { 1 } else { 0 }].y,
                    self[if corner & 4 != 0 { 1 } else { 0 }].y)
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
            if v.x > v.z { Axis::X } else { Axis::Z }
        } else if v.y > v.z {
            Axis::Y
        } else {
            Axis::Z
        }
    }

    pub fn union(bbox1: &Bounds3<T>, bbox2: &Bounds3<T>) -> Bounds3<T> {
        let min = Point3::new(min(bbox1.p_min.x, bbox2.p_min.x),
                              min(bbox1.p_min.y, bbox2.p_min.y),
                              min(bbox1.p_min.z, bbox2.p_min.z));
        let max = Point3::new(max(bbox1.p_max.x, bbox2.p_max.x),
                              max(bbox1.p_max.y, bbox2.p_max.y),
                              max(bbox1.p_max.z, bbox2.p_max.z));

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

    pub fn intersect_p(&self, ray: &mut Ray) -> bool {
        let invdir = Vector::new(1.0 / ray.d.x, 1.0 / ray.d.y, 1.0 / ray.d.z);
        let sign = [(ray.d.x < 0.0) as usize, (ray.d.y < 0.0) as usize, (ray.d.z < 0.0) as usize];

        self.intersect_p_fast(ray, &invdir, &sign)
    }

    pub fn intersect_p_fast(&self,
                            ray: &mut Ray,
                            inv_dir: &Vector,
                            dir_is_neg: &[usize; 3])
                            -> bool {
        stats::inc_fast_bbox_isect();
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

        tmin < ray.t_max && tmax > ray.t_min
    }

    /// Linearly interpolate a point inside the bounds
    pub fn lerp(&self, t: &Point3<T>) -> Point3<T> {
        Point3::new(lerp(t.x, self.p_min.x, self.p_max.x),
                    lerp(t.y, self.p_min.y, self.p_max.y),
                    lerp(t.z, self.p_min.z, self.p_max.z))
    }

    pub fn inside(&self, p: &Point3<T>) -> bool {
        p.x >= self.p_min.x && p.x <= self.p_max.x && p.y >= self.p_min.y &&
        p.y <= self.p_max.y && p.z >= self.p_min.z && p.z <= self.p_max.z
    }
}

impl Bounds3<f32> {
    /// Compute the bounding sphere of the current bounding box, and returns its center and radius.
    pub fn bounding_sphere(&self) -> (Point, f32) {
        let center = Point::new((self.p_min.x + self.p_max.x) / 2.0,
                                (self.p_min.y + self.p_max.y) / 2.0,
                                (self.p_min.z + self.p_max.z) / 2.0);
        let radius = if self.inside(&center) {
            (self.p_max - center).norm()
        } else {
            0.0
        };

        (center, radius)
    }
}

impl<T> Index<usize> for Bounds3<T> {
    type Output = Point3<T>;

    fn index(&self, i: usize) -> &Point3<T> {
        match i {
            0 => &self.p_min,
            1 => &self.p_max,
            _ => panic!("Invalid index!"),
        }
    }
}

fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a.lt(&b) { a } else { b }
}

fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a.gt(&b) { a } else { b }
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
