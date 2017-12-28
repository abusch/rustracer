use std::f32::consts::{FRAC_1_PI, PI};
use std::path::Path;
use std::cmp::min;
use std::sync::Arc;

use num::Zero;
use parking_lot::RwLock;

use {Point2f, Point2i, Point3f, Transform, Vector3f};
use geometry::{spherical_phi, spherical_theta};
use imageio::read_image;
use interaction::Interaction;
use light::{Light, LightFlags, VisibilityTester};
use mipmap::{MIPMap, WrapMode};
use paramset::ParamSet;
use ray::Ray;
use sampling::Distribution2D;
use scene::Scene;
use spectrum::Spectrum;

#[derive(Debug)]
pub struct InfiniteAreaLight {
    id: u32,
    light_to_world: Transform,
    world_to_light: Transform,
    n_samples: u32,
    l_map: Box<MIPMap<Spectrum>>,
    world_center: RwLock<Point3f>,
    world_radius: RwLock<f32>,
    distribution: Box<Distribution2D>,
}

impl InfiniteAreaLight {
    pub fn new<P: AsRef<Path>>(l2w: Transform,
                               n_samples: u32,
                               power: Spectrum,
                               texmap: P)
                               -> InfiniteAreaLight {
        let texmap = texmap.as_ref();
        // Read texel data from texmap and initialise Lmap
        let (resolution, texels) = if let Ok((pixels, res)) = read_image(texmap) {
            info!("Loading environment map {} for infinite light",
                  texmap.display());
            let texels = pixels.iter().map(|s| *s * power).collect();
            (res, texels)
        } else {
            warn!("Environment map {} for infinite light not found! Using constant texture \
                 instead.",
                  texmap.display());
            (Point2i::new(1, 1), vec![power])
        };
        //
        let l_map = Box::new(MIPMap::new(&resolution, &texels[..], false, 0.0, WrapMode::Repeat));
        // initialize sampling PDFs for infinite area light
        // - compute scalar-valued image img from environment map
        let (width, height) = (2 * l_map.width(), 2 * l_map.height());
        let filter = 0.5 / min(width, height) as f32;
        let mut img = Vec::with_capacity(width * height);
        for v in 0..height {
            let vp = (v as f32 + 0.5) / height as f32;
            let sin_theta = (PI * (v as f32 + 0.5) / height as f32).sin();
            for u in 0..width {
                let up = (u as f32 + 0.5) / width as f32;
                img.push(l_map.lookup(&Point2f::new(up, vp), filter).y() * sin_theta);
            }
        }
        // - compute sampling distributions for rows and columns of image
        let distribution = Box::new(Distribution2D::new(&img[..], width, height));

        InfiniteAreaLight {
            id: super::get_next_id(),
            world_to_light: l2w.inverse(),
            light_to_world: l2w,
            n_samples: n_samples,
            l_map: l_map,
            world_center: RwLock::new(Point3f::zero()),
            world_radius: RwLock::new(0.0), // TODO
            distribution: distribution,
        }
    }

    pub fn create(l2w: &Transform, params: &mut ParamSet) -> Arc<Light> {
        let L = params.find_one_spectrum("L", Spectrum::white());
        let scale = params.find_one_spectrum("scale", Spectrum::white());
        let mapname = params.find_one_filename("mapname", "".to_owned());
        let n_samples = params.find_one_int("samples", 1);
        // TODO quickrender
        Arc::new(InfiniteAreaLight::new(l2w.clone(), n_samples as u32, L * scale, mapname))
    }
}

impl Light for InfiniteAreaLight {
    fn id(&self) -> u32 {
        self.id
    }

    fn preprocess(&self, scene: &Scene) {
        let (w_center, w_radius) = scene.world_bounds().bounding_sphere();
        let mut wc = self.world_center.write();
        *wc = w_center;
        let mut wr = self.world_radius.write();
        *wr = w_radius;
    }

    fn sample_li(&self,
                 isect: &Interaction,
                 u: &Point2f)
                 -> (Spectrum, Vector3f, f32, VisibilityTester) {
        // Find (u, v) sample coordinates in infinite light texture
        let (uv, map_pdf) = self.distribution.sample_continuous(u);
        if map_pdf == 0.0 {
            return (Spectrum::black(),
                    Vector3f::new(0.0, 0.0, 0.0),
                    0.0,
                    VisibilityTester::new(Interaction::from_point(&Point3f::zero()),
                                          Interaction::from_point(&Point3f::zero())));
        }
        // Convert infinite light sample point to direction
        let theta = uv[1] * PI;
        let phi = uv[0] * 2.0 * PI;
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();
        let wi = &self.light_to_world *
                 &Vector3f::new(sin_theta * cos_phi, sin_theta * sin_phi, cos_theta);
        // Compute PDF for sampled infinite light direction
        let pdf = if sin_theta == 0.0 {
            0.0
        } else {
            map_pdf / (2.0 * PI * PI * sin_theta)
        };
        // Return radiance value for infinite light direction
        let world_radius = self.world_radius.read();
        let target = isect.p + wi * (2.0 * *world_radius);
        let vis = VisibilityTester::new(*isect, Interaction::from_point(&target));
        (self.l_map.lookup(&uv, 0.0), wi, pdf, vis)
    }

    fn pdf_li(&self, _si: &Interaction, w: &Vector3f) -> f32 {
        let wi = &self.world_to_light * w;
        let theta = spherical_theta(&wi);
        let phi = spherical_phi(&wi);
        let sin_theta = theta.sin();

        if sin_theta == 0.0 {
            0.0
        } else {
            self.distribution
                .pdf(&Point2f::new(phi * FRAC_1_PI * 0.5, theta * FRAC_1_PI)) /
            (2.0 * PI * PI * sin_theta)
        }
    }

    fn n_samples(&self) -> u32 {
        self.n_samples
    }

    fn flags(&self) -> LightFlags {
        LightFlags::INFINITE
    }

    fn power(&self) -> Spectrum {
        let world_radius = self.world_radius.read();
        PI * *world_radius * *world_radius * self.l_map.lookup(&Point2f::new(0.5, 0.5), 0.5)
    }

    fn le(&self, ray: &Ray) -> Spectrum {
        let w = (&self.world_to_light * &ray.d).normalize();
        let st = Point2f::new(spherical_phi(&w) * FRAC_1_PI * 0.5,
                              spherical_theta(&w) * FRAC_1_PI);

        self.l_map.lookup(&st, 0.0)
    }
}
