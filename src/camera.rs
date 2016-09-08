use std::f32::consts::PI;

use Vector;
use Dim;
use Point;
use ray::Ray;
use na::Norm;


pub struct Camera {
    pub origin: Point,
    pub dimension: Dim,
    pub fov: f32,
    inv_width: f32,
    inv_height: f32,
    aspect_ratio: f32,
    angle: f32,
}

impl Camera {
    pub fn new(origin: Point, dims: Dim, fov: f32) -> Camera {
        let (w, h) = dims;
        Camera {
            origin: origin,
            dimension: dims,
            fov: fov,
            inv_width: 1.0 / w as f32,
            inv_height: 1.0 / h as f32,
            aspect_ratio: w as f32 / h as f32,
            angle: (PI * 0.5 * fov / 180.0).tan(),
        }
    }

    pub fn ray_for(&self, x: f32, y: f32) -> Ray {
        let xx = (2.0 * x * self.inv_width - 1.0) * self.angle * self.aspect_ratio;
        let yy = (1.0 - 2.0 * y * self.inv_height) * self.angle;
        let raydir = Vector::new(xx, yy, -1.0).normalize();
        Ray::new(self.origin, raydir)
    }
}
