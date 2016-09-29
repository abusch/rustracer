use Vector;
use Point;
use ray::Ray;
use na::origin;

/// Axis Aligned Bounding Box
pub struct BBox {
    pub bounds: [Point; 2],
}

impl BBox {
    pub fn new() -> BBox {
        BBox { bounds: [origin(), origin()] }
    }

    pub fn intersect(&self, ray: &mut Ray) -> bool {
        let invdir = Vector::new(1.0 / ray.dir.x, 1.0 / ray.dir.y, 1.0 / ray.dir.z);
        let sign =
            [(ray.dir.x < 0.0) as usize, (ray.dir.y < 0.0) as usize, (ray.dir.z < 0.0) as usize];

        let mut tmin = (self.bounds[sign[0]].x - ray.origin.x) * invdir.x;
        let mut tmax = (self.bounds[1 - sign[0]].x - ray.origin.x) * invdir.x;
        let tymin = (self.bounds[sign[1]].y - ray.origin.y) * invdir.y;
        let tymax = (self.bounds[1 - sign[1]].y - ray.origin.y) * invdir.y;

        if (tmin > tymax) || (tymin > tmax) {
            return false;
        }

        if tymin > tmin {
            tmin = tymin;
        }
        if tymax < tmax {
            tmax = tymax;
        }

        let tzmin = (self.bounds[sign[2]].z - ray.origin.z) * invdir.z;
        let tzmax = (self.bounds[1 - sign[2]].z - ray.origin.z) * invdir.z;

        if (tmin > tzmax) || (tzmin > tmax) {
            return false;
        }

        if tzmin > tmin {
            tmin = tzmin;
        }
        if tzmax < tmax {
            tmax = tzmax;
        }

        if tmin < 0.0 && tmax < 0.0 {
            return false;
        }

        true
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
}
