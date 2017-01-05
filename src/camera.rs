use std::f32::consts::PI;

use na::{self, Norm, Inverse, Matrix4, FromHomogeneous, ToHomogeneous, PerspectiveMatrix3};

use {Vector3f, Point3f, Point2f, Transform};
use bounds::Bounds2f;
use ray::Ray;

/// Projective pinhole camera.
/// TODO: abstract the Camera interface and handle multiple camera types.
#[derive(Debug)]
pub struct Camera {
    camera_to_world: Transform,
    camera_to_screen: Matrix4<f32>,
    raster_to_camera: Matrix4<f32>,
}

impl Camera {
    pub fn new(camera_to_world: Transform,
               film_size: Point2f,
               lens_radius: f32,
               focal_distance: f32,
               fov: f32)
               -> Camera {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        fn scale(x: f32, y: f32, z: f32) -> Matrix4<f32> {
            Matrix4::<f32>::new(x, 0.0, 0.0, 0.0,
                                0.0, y, 0.0, 0.0,
                                0.0, 0.0, z, 0.0,
                                0.0, 0.0, 0.0, 1.0)
        }

        #[cfg_attr(rustfmt, rustfmt_skip)]
        fn translate(x: f32, y: f32, z: f32) -> Matrix4<f32> {
            Matrix4::<f32>::new(1.0, 0.0, 0.0, x,
                                0.0, 1.0, 0.0, y,
                                0.0, 0.0, 1.0, z,
                                0.0, 0.0, 0.0, 1.0)
        }

        #[cfg_attr(rustfmt, rustfmt_skip)]
        fn perspective(fov: f32, n: f32, f: f32) -> Matrix4<f32> {
            let persp = Matrix4::<f32>::new(1.0, 0.0, 0.0, 0.0,
                                            0.0, 1.0, 0.0, 0.0,
                                            0.0, 0.0, f / (f - n), -f * n / (f - n),
                                            0.0, 0.0, 1.0, 0.0);
            let inv_tan_ang = 1.0 / (fov.to_radians() / 2.0).tan();
            scale(inv_tan_ang, inv_tan_ang, 1.0) * persp
        }

        let aspect_ratio = film_size.x / film_size.y;
        let screen_window = if aspect_ratio > 1.0 {
            Bounds2f::from_points(&Point2f::new(-aspect_ratio, -1.0),
                                  &Point2f::new(aspect_ratio, 1.0))
        } else {
            Bounds2f::from_points(&Point2f::new(-1.0, -1.0 / aspect_ratio),
                                  &Point2f::new(1.0, 1.0 / aspect_ratio))
        };
        let camera_to_screen = perspective(fov, 1e-2, 1000.0);
        // PerspectiveMatrix3::<f32>::new(1.0, fov.to_radians(), 1e-2, 1000.0);
        // let camera_to_screen = camera_to_screen_perspective.as_matrix();
        let screen_to_raster = scale(film_size.x, film_size.y, 1.0) *
                               scale(1.0 / (screen_window.p_max.x - screen_window.p_min.x),
                                     1.0 / (screen_window.p_min.y - screen_window.p_max.y),
                                     1.0) *
                               translate(-screen_window.p_min.x, -screen_window.p_max.y, 0.0);
        let raster_to_screen = screen_to_raster.inverse().unwrap();
        let raster_to_camera = camera_to_screen.inverse().unwrap() * raster_to_screen;
        info!("camera_to_world={}", camera_to_world);
        Camera {
            camera_to_world: camera_to_world,
            camera_to_screen: camera_to_screen,
            raster_to_camera: raster_to_camera,
        }
    }

    pub fn generate_ray(&self, sample: &Point2f) -> Ray {
        let p_film = Point3f::new(sample.x, sample.y, 0.0);
        let p_camera: Point3f = na::from_homogeneous(&(self.raster_to_camera *
                                                       p_film.to_homogeneous()));

        let ray = Ray::new(na::origin(), p_camera.to_vector().normalize());
        // TODO modify ray for depth of field
        ray.transform(&self.camera_to_world).0
    }
}

pub struct CameraSample {
    pub p_film: Point2f,
    pub p_lens: Point2f,
    pub time: f32,
}
