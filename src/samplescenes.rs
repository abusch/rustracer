use std::path::Path;
use std::sync::Arc;

use rt::camera::Camera;
use rt::light::{Light, InfiniteAreaLight};
use rt::material::matte::MatteMaterial;
use rt::material::glass::GlassMaterial;
use rt::primitive::{Primitive, GeometricPrimitive};
use rt::scene::Scene;
use rt::shapes::disk::Disk;
use rt::shapes::sphere::Sphere;
use rt::spectrum::Spectrum;
use rt::{Transform, Dim, Point2f};

pub fn build_scene(dim: Dim) -> Scene {
    info!("Building scene");
    let camera = Camera::new(Transform::translate_z(-3.0),
                             Point2f::new(dim.0 as f32, dim.1 as f32),
                             0.00,
                             2.5,
                             60.0);
    let mut lights: Vec<Arc<Light + Send + Sync>> = Vec::new();

    // let disk = Arc::new(Disk::new(-2.0, 0.8, 0.0, 360.0, transform::rot_x(90.0)));
    // let area_light =
    //     Arc::new(DiffuseAreaLight::new(Spectrum::rgb(1.0, 1.0, 1.0), disk.clone(), 16));
    // let area_light_prim = Box::new(GeometricPrimitive {
    //     shape: disk.clone(),
    //     area_light: Some(area_light.clone()),
    //     material: Some(Arc::new(MatteMaterial::default())),
    // });

    // let bronze = Arc::new(Metal::new());
    // let gold = Arc::new(Metal::gold());
    // let plastic = Arc::new(Plastic::new(Spectrum::rgb(0.3, 0.3, 1.0), Spectrum::white()));
    // let plastic_white = Arc::new(Plastic::new(Spectrum::white(), Spectrum::white()));
    // let plastic_lines = Arc::new(Plastic::new_tex("grid.png", Spectrum::white()));
    // let plastic_lines = Arc::new(MatteMaterial::new_uv_texture());
    let glass = Arc::new(GlassMaterial::new().roughness(0.00, 0.00));
    let matte_red = Arc::new(MatteMaterial::new(Spectrum::rgb(1.0, 0.0, 0.0), 0.0));
    let sphere = Box::new(GeometricPrimitive {
                              shape: Arc::new(Sphere::new()
                                                  .radius(0.7)
                                                  .transform(Transform::translate_y(-0.3))),
                              area_light: None,
                              material: Some(glass.clone()),
                          });
    // let bunny =
    //     Box::new(BVH::<GeometricPrimitive>::from_mesh_file(Path::new("models/bunny.obj"),
    //                                                        "bunny",
    //                                                        plastic.clone(),
    //                                                        &Transform::new(
    //                                                          Vector3f::new(2.0, -0.8, 0.0),
    //                                                          Vector3f::new(0.0, 20.0f32.to_radians(), 0.0),
    //                                                          0.5
    //                                                          ))) as Box<Primitive + Send + Sync>;
    // let buddha =
    //     Box::new(BVH::<GeometricPrimitive>::from_mesh_file(Path::new("models/buddha.obj"),
    //                                                        "buddha",
    //                                                        gold.clone(),
    //                                                        &Transform::new(
    //                                                          Vector3f::new(-2.0, 0.0, 0.0),
    //                                                          Vector3f::new(0.0, 0.0, 0.0),
    //                                                          2.0
    //                                                          ))) as Box<Primitive + Send + Sync>;
    // let dragon =
    //     Box::new(BVH::<GeometricPrimitive>::from_mesh_file(Path::new("models/dragon.obj"),
    //                                                        "dragon",
    //                                                        gold.clone(),
    //                                                        &Transform::new(
    //                                                          Vector3f::new(-0.2, 0.0, 0.0),
    //                                                          Vector3f::new(0.0, -70.0f32.to_radians(), 0.0),
    //                                                          3.0
    //                                                          ))) as Box<Primitive + Send + Sync>;
    let floor =
        Box::new(GeometricPrimitive {
                     shape: Arc::new(Disk::new(-1.0, 20.0, 0.0, 360.0, Transform::rot_x(-90.0))),
                     area_light: None,
                     material: Some(matte_red.clone()),
                 });

    let primitives: Vec<Box<Primitive + Sync + Send>> = vec![sphere, floor];
    // Light
    // lights.push(area_light);
    // lights.push(Arc::new(DistantLight::new(Vector3f::new(0.0, -1.0, 5.0),
    //                                        Spectrum::rgb(1.0, 1.0, 1.0))));
    lights.push(Arc::new(InfiniteAreaLight::new(Transform::rot_x(-90.0),
                                                16,
                                                Spectrum::grey(1.0),
                                                Path::new("RenoSuburb01_sm.exr"))));
    // Path::new("sky_sanmiguel.tga"))));

    Scene::new(camera, primitives, lights)
}
