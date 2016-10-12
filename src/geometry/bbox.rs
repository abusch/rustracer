use std::f32;
use std::ops::Index;
use Vector;
use Point;
use ray::Ray;
use stats;
use na::origin;

/// Axis Aligned Bounding Box
#[derive(Debug, Copy, Clone)]
pub struct BBox {
    pub bounds: [Point; 2],
}

impl BBox {
    pub fn new() -> BBox {
        BBox {
            bounds: [Point::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
                     Point::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY)],
        }
    }

    pub fn from_point(point: &Point) -> BBox {
        BBox { bounds: [*point, *point] }
    }

    pub fn intersect(&self, ray: &mut Ray) -> bool {
        let invdir = Vector::new(1.0 / ray.dir.x, 1.0 / ray.dir.y, 1.0 / ray.dir.z);
        let sign =
            [(ray.dir.x < 0.0) as usize, (ray.dir.y < 0.0) as usize, (ray.dir.z < 0.0) as usize];

        self.intersect_p(ray, &invdir, &sign)

        // let mut tmin = (self.bounds[sign[0]].x - ray.origin.x) * invdir.x;
        // let mut tmax = (self.bounds[1 - sign[0]].x - ray.origin.x) * invdir.x;
        // let tymin = (self.bounds[sign[1]].y - ray.origin.y) * invdir.y;
        // let tymax = (self.bounds[1 - sign[1]].y - ray.origin.y) * invdir.y;

        // if (tmin > tymax) || (tymin > tmax) {
        //     return false;
        // }

        // if tymin > tmin {
        //     tmin = tymin;
        // }
        // if tymax < tmax {
        //     tmax = tymax;
        // }

        // let tzmin = (self.bounds[sign[2]].z - ray.origin.z) * invdir.z;
        // let tzmax = (self.bounds[1 - sign[2]].z - ray.origin.z) * invdir.z;

        // if (tmin > tzmax) || (tzmin > tmax) {
        //     return false;
        // }

        // if tzmin > tmin {
        //     tmin = tzmin;
        // }
        // if tzmax < tmax {
        //     tmax = tzmax;
        // }

        // if tmin < 0.0 && tmax < 0.0 {
        //     return false;
        // }

        // true
    }

    pub fn intersect_p(&self, ray: &mut Ray, inv_dir: &Vector, dir_is_neg: &[usize; 3]) -> bool {
        // stats::inc_fast_bbox_isect();
        // Check intersection with X and Y slab
        let mut tmin = (self.bounds[dir_is_neg[0]].x - ray.origin.x) * inv_dir.x;
        let mut tmax = (self.bounds[1 - dir_is_neg[0]].x - ray.origin.x) * inv_dir.x;
        let tymin = (self.bounds[dir_is_neg[1]].y - ray.origin.y) * inv_dir.y;
        let tymax = (self.bounds[1 - dir_is_neg[1]].y - ray.origin.y) * inv_dir.y;
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
        let tzmin = (self.bounds[dir_is_neg[2]].z - ray.origin.z) * inv_dir.z;
        let tzmax = (self.bounds[1 - dir_is_neg[2]].z - ray.origin.z) * inv_dir.z;
        if (tmin > tzmax) || (tzmin > tmax) {
            return false;
        }
        if tzmin > tmin {
            tmin = tzmin;
        }
        if tzmax < tmax {
            tmax = tzmax;
        }

        return tmin < ray.t_max && tmax > ray.t_min;
    }

    pub fn extend(&mut self, p: &Point) {
        if p.x < self.bounds[0].x {
            self.bounds[0].x = p.x
        }
        if p.y < self.bounds[0].y {
            self.bounds[0].y = p.y
        }
        if p.z < self.bounds[0].z {
            self.bounds[0].z = p.z
        }
        if p.x > self.bounds[1].x {
            self.bounds[1].x = p.x
        }
        if p.y > self.bounds[1].y {
            self.bounds[1].y = p.y
        }
        if p.z > self.bounds[1].z {
            self.bounds[1].z = p.z
        }
    }

    pub fn maximum_extent(&self) -> Axis {
        let v = self.bounds[1] - self.bounds[0];
        if v.x > v.y {
            if v.x > v.z { Axis::X } else { Axis::Z }
        } else {
            if v.y > v.z { Axis::Y } else { Axis::Z }
        }
    }


    pub fn union(bbox1: &BBox, bbox2: &BBox) -> BBox {
        let min = Point::new(bbox1.bounds[0].x.min(bbox2.bounds[0].x),
                             bbox1.bounds[0].y.min(bbox2.bounds[0].y),
                             bbox1.bounds[0].z.min(bbox2.bounds[0].z));
        let max = Point::new(bbox1.bounds[1].x.max(bbox2.bounds[1].x),
                             bbox1.bounds[1].y.max(bbox2.bounds[1].y),
                             bbox1.bounds[1].z.max(bbox2.bounds[1].z));

        BBox { bounds: [min, max] }
    }

    pub fn union_point(bbox: &BBox, p: &Point) -> BBox {
        let mut b = *bbox;
        b.extend(p);
        b
    }
}

pub trait Bounded {
    fn get_world_bounds(&self) -> BBox;
}

#[derive(Copy, Clone, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Index<Axis> for Point {
    type Output = f32;

    fn index(&self, axis: Axis) -> &f32 {
        match axis {
            Axis::X => &self.x,
            Axis::Y => &self.y,
            Axis::Z => &self.z,
        }
    }
}
