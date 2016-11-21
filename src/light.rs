use std::f32;
use std::f32::consts::*;

use na::{Dot, Norm};

use {Point, Point2f, Vector};
use interaction::SurfaceInteraction;
use ray::Ray;
use scene::Scene;
use shapes::Shape;
use spectrum::Spectrum;

bitflags! {
    pub flags LightFlags: u32 {
        const DELTA_POSITION  = 0b_00000001,
        const DELTA_DIRECTION = 0b_00000010,
        const AREA            = 0b_00000100,
        const INFINITE        = 0b_00001000,
    }
}

#[inline]
fn is_delta_light(flags: LightFlags) -> bool {
    flags.contains(DELTA_POSITION) || flags.contains(DELTA_DIRECTION)
}

pub struct VisibilityTester {
    ray: Ray,
}

impl VisibilityTester {
    pub fn new(ray: Ray) -> VisibilityTester {
        VisibilityTester { ray: ray }
    }

    pub fn unoccluded(&self, scene: &Scene) -> bool {
        !scene.intersect_p(&self.ray)
    }
}

pub trait Light {
    /// Sample the light source for an outgoing direction wo.
    /// Return a triplet of:
    ///  * emitted light in the sampled direction
    ///  * the sampled direction wi
    ///  * the pdf for that direction
    ///  * A VisibilityTester
    fn sample_li(&self,
                 isect: &SurfaceInteraction,
                 wo: &Vector,
                 u: &Point2f)
                 -> (Spectrum, Vector, f32, VisibilityTester);

    fn pdf_li(&self, si: &SurfaceInteraction, wi: &Vector) -> f32;

    fn preprocess(&mut self, scene: &Scene) {}

    fn n_samples(&self) -> u32;

    fn flags(&self) -> LightFlags;

    fn power(&self) -> Spectrum;
}

#[derive(Debug)]
pub struct PointLight {
    pub pos: Point,
    pub emission_colour: Spectrum,
}

impl PointLight {
    pub fn new(p: Point, ec: Spectrum) -> PointLight {
        PointLight {
            pos: p,
            emission_colour: ec,
        }
    }
}

impl Light for PointLight {
    fn sample_li(&self,
                 isect: &SurfaceInteraction,
                 wo: &Vector,
                 u: &Point2f)
                 -> (Spectrum, Vector, f32, VisibilityTester) {
        let wi = self.pos - isect.p;
        let r2 = wi.norm_squared();
        let l_i = self.emission_colour / (4.0 * PI * r2);
        let vt = VisibilityTester::new(isect.spawn_ray_to(&self.pos));

        (l_i, wi.normalize(), 1.0, vt)
    }

    fn pdf_li(&self, si: &SurfaceInteraction, _wi: &Vector) -> f32 {
        0.0
    }

    fn n_samples(&self) -> u32 {
        1
    }

    fn flags(&self) -> LightFlags {
        DELTA_POSITION
    }

    fn power(&self) -> Spectrum {
        4.0 * PI * self.emission_colour
    }
}

#[derive(Debug)]
pub struct DistantLight {
    pub dir: Vector,
    pub emission_colour: Spectrum,
    w_center: Point,
    w_radius: f32,
}

impl DistantLight {
    pub fn new(dir: Vector, ec: Spectrum) -> DistantLight {
        DistantLight {
            dir: dir.normalize(),
            emission_colour: ec,
            w_center: Point::new(0.0, 0.0, 0.0),
            w_radius: 0.0,
        }
    }
}

impl Light for DistantLight {
    fn preprocess(&mut self, scene: &Scene) {
        let (w_center, w_radius) = scene.world_bounds().bounding_sphere();
        self.w_center = w_center;
        self.w_radius = w_radius;
    }

    fn sample_li(&self,
                 isect: &SurfaceInteraction,
                 _wo: &Vector,
                 _u: &Point2f)
                 -> (Spectrum, Vector, f32, VisibilityTester) {
        let p_outside = isect.p - self.dir * (2.0 * self.w_radius);
        (self.emission_colour,
         -self.dir,
         1.0,
         VisibilityTester::new(isect.spawn_ray_to(&p_outside)))
    }

    fn pdf_li(&self, si: &SurfaceInteraction, wi: &Vector) -> f32 {
        0.0
    }

    fn n_samples(&self) -> u32 {
        1
    }

    fn flags(&self) -> LightFlags {
        DELTA_DIRECTION
    }

    fn power(&self) -> Spectrum {
        self.emission_colour * PI * self.w_radius * self.w_radius
    }
}

pub trait AreaLight: Light {
    // TODO SurfaceInteraction should be Interation to support mediums
    fn l(&self, si: &SurfaceInteraction, w: &Vector) -> Spectrum;
}

pub struct DiffuseAreaLight {
    l_emit: Spectrum,
    shape: Box<Shape + Send + Sync>,
    n_samples: u32,
    area: f32,
}

impl DiffuseAreaLight {}

impl Light for DiffuseAreaLight {
    fn sample_li(&self,
                 si: &SurfaceInteraction,
                 wo: &Vector,
                 u: &Point2f)
                 -> (Spectrum, Vector, f32, VisibilityTester) {
        let p_shape = self.shape.sample_si(si, u);
        let wi = (p_shape.p - si.p).normalize();
        let pdf = self.shape.pdf_wi(si, &wi);
        let vis = VisibilityTester::new(si.spawn_ray_to(&p_shape.p));

        (self.l(si, &(-wi)), wi, pdf, vis)
    }

    fn pdf_li(&self, si: &SurfaceInteraction, wi: &Vector) -> f32 {
        self.shape.pdf_wi(si, wi)
    }

    fn n_samples(&self) -> u32 {
        self.n_samples
    }

    fn flags(&self) -> LightFlags {
        AREA
    }

    fn power(&self) -> Spectrum {
        self.l_emit * PI * self.area
    }
}

impl AreaLight for DiffuseAreaLight {
    fn l(&self, si: &SurfaceInteraction, w: &Vector) -> Spectrum {
        if si.n.dot(w) > 0.0 {
            self.l_emit
        } else {
            Spectrum::black()
        }
    }
}
