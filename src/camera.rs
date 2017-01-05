use std::f32::consts::PI;

use na::{Norm, Inverse};

use {Vector, Dim, Point, Point2f, Transform};
use ray::Ray;

/// Projective pinhole camera.
/// TODO: abstract the Camera interface and handle multiple camera types.
pub struct Camera {
    camera_to_world: Transform,
    camera_to_screen: Transform,
    raster_to_camera: Transform, /* pub origin: Point,
                                  * pub dimension: Dim,
                                  * pub fov: f32,
                                  * inv_width: f32,
                                  * inv_height: f32,
                                  * aspect_ratio: f32,
                                  * angle: f32, */
}

impl Camera {
    pub fn new(camera_to_world: Transform,
               camera_to_screen: Transform,
               screen_window: Point2f,
               lensr: f32,
               focald: f32)
               -> Camera {

       let screen_to_raster = ...;
       let raster_to_screen = screen_to_raster.inverse().unwrap();
        let raster_to_camera = camera_to_screen.inverse().unwrap() * raster_to_screen;
        Camera { 
            camera_to_world: camera_to_world,
            camera_to_screen: camera_to_screen,
        }
    }

    pub fn generate_ray(&self, sample: CameraSample) -> Ray {
        unimplemented!();
    }
    // pub fn new(origin: Point, dims: Dim, fov: f32) -> Camera {
    //     let (w, h) = dims;
    //     Camera {
    //         origin: origin,
    //         dimension: dims,
    //         fov: fov,
    //         inv_width: 1.0 / w as f32,
    //         inv_height: 1.0 / h as f32,
    //         aspect_ratio: w as f32 / h as f32,
    //         angle: (PI * 0.5 * fov / 180.0).tan(),
    //     }
    // }

    // pub fn ray_for(&self, p: &Point2f) -> Ray {
    //     let xx = (2.0 * p.x * self.inv_width - 1.0) * self.angle * self.aspect_ratio;
    //     let yy = (1.0 - 2.0 * p.y * self.inv_height) * self.angle;
    //     let raydir = Vector::new(xx, yy, 1.0).normalize();
    //     Ray::new(self.origin, raydir)
    // }
}

pub struct CameraSample {
    pub p_film: Point2f,
    pub p_lens: Point2f,
    pub time: f32,
}
