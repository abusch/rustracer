use na::{self, Matrix4, Point3, Vector3};

use {Vector3f, Point3f, Point2f, Transform};
use bounds::Bounds2f;
use film::Film;
use paramset::ParamSet;
use ray::{Ray, RayDifferential};
use sampling;

pub trait Camera {
    fn get_film(&self) -> &Box<Film>;
    fn generate_ray(&self, sample: &CameraSample) -> Ray;
    fn generate_ray_differential(&self, sample: &CameraSample) -> Ray;
}

/// Projective pinhole camera.
pub struct PerspectiveCamera {
    film: Box<Film>,
    camera_to_world: Transform,
    camera_to_screen: Matrix4<f32>,
    raster_to_camera: Matrix4<f32>,
    lens_radius: f32,
    focal_distance: f32,
    dx_camera: Vector3f,
    dy_camera: Vector3f,
}

impl PerspectiveCamera {
    pub fn new(camera_to_world: Transform,
               screen_window: Bounds2f,
               lens_radius: f32,
               focal_distance: f32,
               fov: f32,
               film: Box<Film>)
               -> PerspectiveCamera {

        #[cfg_attr(rustfmt, rustfmt_skip)]
        fn perspective(fov: f32, n: f32, f: f32) -> Matrix4<f32> {
            let persp = Matrix4::<f32>::new(1.0, 0.0, 0.0, 0.0,
                                            0.0, 1.0, 0.0, 0.0,
                                            0.0, 0.0, f / (f - n), -f * n / (f - n),
                                            0.0, 0.0, 1.0, 0.0);
            let inv_tan_ang = 1.0 / (fov.to_radians() / 2.0).tan();
            Matrix4::new_nonuniform_scaling(&Vector3f::new(inv_tan_ang, inv_tan_ang, 1.0)) * persp
        }

        let camera_to_screen = perspective(fov, 1e-2, 1000.0);
        let screen_to_raster =
            Matrix4::new_nonuniform_scaling(&Vector3f::new(film.full_resolution.x as f32,
                                                           film.full_resolution.y as f32,
                                                           1.0)) *
            Matrix4::new_nonuniform_scaling(&Vector3f::new(1.0 /
                                                           (screen_window.p_max.x -
                                                            screen_window.p_min.x),
                                                           1.0 /
                                                           (screen_window.p_min.y -
                                                            screen_window.p_max.y),
                                                           1.0)) *
            Matrix4::new_translation(&Vector3f::new(-screen_window.p_min.x,
                                                    -screen_window.p_max.y,
                                                    0.0));
        let raster_to_screen = screen_to_raster.try_inverse().unwrap();
        let raster_to_camera = camera_to_screen.try_inverse().unwrap() * raster_to_screen;

        // compute differential changes in origin for perspective camera rays
        let dx_camera = Vector3::from_homogeneous((raster_to_camera *
                                                   Point3f::new(1.0, 0.0, 0.0).to_homogeneous()) -
                                                  (raster_to_camera *
                                                   Point3f::new(0.0, 0.0, 0.0).to_homogeneous()))
                .unwrap();
        let dy_camera = Vector3::from_homogeneous((raster_to_camera *
                                                   Point3f::new(0.0, 1.0, 0.0).to_homogeneous()) -
                                                  (raster_to_camera *
                                                   Point3f::new(0.0, 0.0, 0.0).to_homogeneous()))
                .unwrap();

        PerspectiveCamera {
            film: film,
            camera_to_world: camera_to_world,
            camera_to_screen: camera_to_screen,
            raster_to_camera: raster_to_camera,
            lens_radius: lens_radius,
            focal_distance: focal_distance,
            dx_camera: dx_camera,
            dy_camera: dy_camera,
        }
    }

    pub fn create(ps: &mut ParamSet, cam2world: &Transform, film: Box<Film>) -> Box<Camera> {
        let mut shutteropen = ps.find_one_float("shutteropen", 0.0);
        let mut shutterclose = ps.find_one_float("shutterclose", 1.0);
        if shutterclose < shutteropen {
            warn!("Shutter close time {} < shutter open time {}.  Swapping them.",
                  shutterclose,
                  shutteropen);
            ::std::mem::swap(&mut shutteropen, &mut shutterclose);
        }
        let lensradius = ps.find_one_float("lensradius", 0.0);
        let focaldistance = ps.find_one_float("focaldistance", 1e6);
        let frame = ps.find_one_float("frameaspectratio",
                                      (film.full_resolution.x as f32 /
                                       film.full_resolution.y as f32));
        let mut screen = if frame > 1.0 {
            Bounds2f::from_points(&Point2f::new(-frame, -1.0), &Point2f::new(frame, 1.0))
        } else {
            Bounds2f::from_points(&Point2f::new(-1.0, -1.0 / frame),
                                  &Point2f::new(1.0, 1.0 / frame))
        };
        if let Some(sw) = ps.find_float("screenwindow") {
            if sw.len() == 4 {
                screen.p_min.x = sw[0];
                screen.p_max.x = sw[1];
                screen.p_min.y = sw[2];
                screen.p_max.y = sw[3];
            } else {
                error!("\"screenwindow\" should have 4 values");
            }
        }
        let mut fov = ps.find_one_float("fov", 90.0);
        let halffov = ps.find_one_float("halffov", -1.0);
        if halffov > 0.0 {
            // hack for structure synth, which exports half of the full fov
            fov = halffov * 2.0;
        }

        Box::new(PerspectiveCamera::new(cam2world.clone(),
                                        screen,
                                        lensradius,
                                        focaldistance,
                                        fov,
                                        film))
    }
}

impl Camera for PerspectiveCamera {
    fn get_film(&self) -> &Box<Film> {
        &self.film
    }

    fn generate_ray(&self, sample: &CameraSample) -> Ray {
        let p_film = Point3f::new(sample.p_film.x, sample.p_film.y, 0.0);
        let p_camera: Point3f =
            Point3::from_homogeneous(self.raster_to_camera * p_film.to_homogeneous()).unwrap();

        let mut ray = Ray::new(na::origin(), p_camera.coords.normalize());
        // modify ray for depth of field
        if self.lens_radius > 0.0 {
            // Sample point on lens
            let p_lens = self.lens_radius * sampling::concentric_sample_disk(&sample.p_lens);
            // Compute point on plane of focus
            let ft = self.focal_distance / ray.d.z;
            let p_focus = ray.at(ft);
            // Update ray for effect of lens
            ray.o = Point3f::new(p_lens.x, p_lens.y, 0.0);
            ray.d = (p_focus - ray.o).normalize();
        }
        ray.transform(&self.camera_to_world).0
    }

    fn generate_ray_differential(&self, sample: &CameraSample) -> Ray {
        let p_film = Point3f::new(sample.p_film.x, sample.p_film.y, 0.0);
        let p_camera: Point3f =
            Point3::from_homogeneous(self.raster_to_camera * p_film.to_homogeneous()).unwrap();

        let mut ray = Ray::new(na::origin(), p_camera.coords.normalize());
        // modify ray for depth of field
        if self.lens_radius > 0.0 {
            // Sample point on lens
            let p_lens = self.lens_radius * sampling::concentric_sample_disk(&sample.p_lens);
            // Compute point on plane of focus
            let ft = self.focal_distance / ray.d.z;
            let p_focus = ray.at(ft);
            // Update ray for effect of lens
            ray.o = Point3f::new(p_lens.x, p_lens.y, 0.0);
            ray.d = (p_focus - ray.o).normalize();
        }
        // compute offset rays for PerspectiveCamera ray differentials
        let diff = if self.lens_radius > 0.0 {
            // Sample point on lens
            let p_lens = self.lens_radius * sampling::concentric_sample_disk(&sample.p_lens);
            let origin = Point3f::new(p_lens.x, p_lens.y, 0.0);

            // ray differential in x direction
            let dx = (p_camera + self.dx_camera).coords.normalize();
            let ft_x = self.focal_distance / dx.z;
            let p_focus_x = Point3::from_coordinates(ft_x * dx);
            let rx_dir = (p_focus_x - origin).normalize();

            // ray differential in x direction
            let dy = (p_camera + self.dy_camera).coords.normalize();
            let ft_y = self.focal_distance / dy.z;
            let p_focus_y = Point3::from_coordinates(ft_y * dy);
            let ry_dir = (p_focus_y - origin).normalize();

            RayDifferential {
                rx_origin: origin,
                ry_origin: origin,
                rx_direction: rx_dir,
                ry_direction: ry_dir,
            }
        } else {
            RayDifferential {
                rx_origin: ray.o,
                ry_origin: ray.o,
                rx_direction: (p_camera.coords + self.dx_camera).normalize(),
                ry_direction: (p_camera.coords + self.dy_camera).normalize(),
            }
        };

        ray.differential = Some(diff);

        ray.transform(&self.camera_to_world).0
    }
}

pub struct CameraSample {
    pub p_film: Point2f,
    pub p_lens: Point2f,
    pub time: f32,
}
