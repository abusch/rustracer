use std::fmt::Debug;
use std::sync::Arc;

use light_arena::Allocator;

use crate::bounds::Bounds3f;
use crate::interaction::SurfaceInteraction;
use crate::light::AreaLight;
use crate::material::{Material, TransportMode};
use crate::ray::Ray;
use crate::shapes::Shape;
use crate::Transform;

pub trait Primitive: Debug + Send + Sync {
    fn world_bounds(&self) -> Bounds3f;

    fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction<'_, '_>>;

    fn intersect_p(&self, ray: &Ray) -> bool;

    fn area_light(&self) -> Option<Arc<dyn AreaLight>>;

    fn material(&self) -> Option<Arc<dyn Material>>;
    fn compute_scattering_functions<'a, 'b>(
        &self,
        isect: &mut SurfaceInteraction<'a, 'b>,
        mode: TransportMode,
        allow_multiple_lobes: bool,
        arena: &'b Allocator<'_>,
    );
}

#[derive(Debug)]
pub struct GeometricPrimitive {
    pub shape: Arc<dyn Shape>,
    pub area_light: Option<Arc<dyn AreaLight>>,
    pub material: Option<Arc<dyn Material>>,
}

impl Primitive for GeometricPrimitive {
    fn world_bounds(&self) -> Bounds3f {
        self.shape.world_bounds()
    }

    fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction<'_, '_>> {
        self.shape.intersect(ray).map(|(mut isect, t_hit)| {
            isect.primitive = Some(self);
            ray.t_max = t_hit;
            isect
        })
    }

    fn intersect_p(&self, ray: &Ray) -> bool {
        self.shape.intersect_p(ray)
    }

    fn area_light(&self) -> Option<Arc<dyn AreaLight>> {
        self.area_light.clone()
    }

    fn material(&self) -> Option<Arc<dyn Material>> {
        self.material.clone()
    }

    fn compute_scattering_functions<'a, 'b>(
        &self,
        isect: &mut SurfaceInteraction<'a, 'b>,
        mode: TransportMode,
        allow_multiple_lobes: bool,
        arena: &'b Allocator<'_>,
    ) {
        if let Some(ref material) = self.material() {
            material.compute_scattering_functions(isect, mode, allow_multiple_lobes, arena);
        }
    }
}

#[derive(Debug)]
pub struct TransformedPrimitive {
    pub primitive: Arc<dyn Primitive>,
    pub primitive_to_world: Transform,
}

impl Primitive for TransformedPrimitive {
    fn world_bounds(&self) -> Bounds3f {
        &self.primitive_to_world * &self.primitive.world_bounds()
    }

    fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction<'_, '_>> {
        let mut r = self.primitive_to_world.inverse() * *ray;
        self.primitive.intersect(&mut r).map(|isect| {
            ray.t_max = r.t_max;
            isect.transform(&self.primitive_to_world)
        })
    }

    fn intersect_p(&self, ray: &Ray) -> bool {
        let r = self.primitive_to_world.inverse() * *ray;
        self.primitive.intersect_p(&r)
    }

    fn area_light(&self) -> Option<Arc<dyn AreaLight>> {
        None
    }

    fn material(&self) -> Option<Arc<dyn Material>> {
        None
    }
    fn compute_scattering_functions<'a, 'b>(
        &self,
        _isect: &mut SurfaceInteraction<'a, 'b>,
        _mode: TransportMode,
        _allow_multiple_lobes: bool,
        _arena: &'b Allocator<'_>,
    ) {
        panic!("TransformedPrimitive::compute_scattering_functions() should not be called!");
    }
}
