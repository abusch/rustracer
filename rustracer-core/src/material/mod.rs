use std::fmt::Debug;
use std::sync::Arc;

use light_arena::Allocator;

use {Normal3f, Vector2f, Vector3f};
use interaction::SurfaceInteraction;
use texture::Texture;

mod matte;
mod metal;
mod plastic;
mod glass;
mod mirror;
mod substrate;
mod translucent;
mod uber;

pub use self::matte::MatteMaterial;
pub use self::metal::Metal;
pub use self::plastic::Plastic;
pub use self::glass::GlassMaterial;
pub use self::mirror::MirrorMaterial;
pub use self::substrate::SubstrateMaterial;
pub use self::translucent::TranslucentMaterial;
pub use self::uber::UberMaterial;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TransportMode {
    RADIANCE,
    IMPORTANCE,
}

pub trait Material: Debug {
    fn compute_scattering_functions<'a, 'b>(&self,
                                            isect: &mut SurfaceInteraction<'a, 'b>,
                                            mode: TransportMode,
                                            allow_multiple_lobes: bool,
                                            arena: &'b Allocator);
}


pub fn bump(d: &Arc<Texture<f32> + Send + Sync>, si: &mut SurfaceInteraction) {
    // Compute offset positions and evaluate displacement texture
    let mut si_eval = si.clone();

    // Shift si_eval du in the u direction
    let mut du = 0.5 * (si.dudx.abs() + si.dudy.abs());
    // The most common reason for du to be zero is for ray that start from
    // light sources, where no differentials are available. In this case,
    // we try to choose a small enough du so that we still get a decently
    // accurate bump value.
    if du == 0.0 {
        du = 0.0005;
    }
    si_eval.p = si.p + du * si.shading.dpdu;
    si_eval.uv = si.uv + Vector2f::new(du, 0.0);
    si_eval.n = (Normal3f::from(si.shading.dpdu.cross(&si.shading.dpdv)) + du * si.dndu)
        .normalize();
    let u_displace = d.evaluate(&si_eval);

    // Shift si_eval dv in the v direction
    let mut dv = 0.5 * (si.dvdx.abs() + si.dvdy.abs());
    if dv == 0.0 {
        dv = 0.0005;
    }
    si_eval.p = si.p + dv * si.shading.dpdv;
    si_eval.uv = si.uv + Vector2f::new(0.0, dv);
    si_eval.n = (Normal3f::from(si.shading.dpdu.cross(&si.shading.dpdv)) + dv * si.dndv)
        .normalize();
    let v_displace = d.evaluate(&si_eval);

    let displace = d.evaluate(si);

    // Compute bump-mapped differential geometry
    let dpdu = si.shading.dpdu + (u_displace - displace) / du * Vector3f::from(si.shading.n) +
               displace * Vector3f::from(si.shading.dndu);
    let dpdv = si.shading.dpdv + (v_displace - displace) / dv * Vector3f::from(si.shading.n) +
               displace * Vector3f::from(si.shading.dndv);
    let dndu = si.shading.dndu;
    let dndv = si.shading.dndv;
    si.set_shading_geometry(&dpdu, &dpdv, &dndu, &dndv, false);
}
