use std::sync::Arc;
use colour::Colourf;
use interaction::SurfaceInteraction;

pub mod matte;

pub enum TransportMode {
    RADIANCE,
    IMPORTANCE,
}

pub trait Material {
    fn compute_scattering_functions(&self,
                                    isect: &mut SurfaceInteraction,
                                    mode: TransportMode,
                                    allow_multiple_lobes: bool);
}
