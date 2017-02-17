use Vector2f;
use interaction::SurfaceInteraction;
use texture::Texture;

pub mod matte;
pub mod metal;
pub mod plastic;
pub mod glass;

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


pub fn bump(d: &Box<Texture<f32> + Send + Sync>, si: &mut SurfaceInteraction) {
    let mut si_eval = si.clone();
    // Shift si du in the du direction
    let mut du = 0.5 * (si.dudx.abs() + si.dudy.abs());
    if du == 0.0 {
        du = 0.0005;
    }
    si_eval.p = si.p + du * si.shading.dpdu;
    si_eval.uv = si.uv + Vector2f::new(du, 0.0);
    si_eval.n = (si.shading.dpdu.cross(&si.shading.dpdv) + du * si.dndu).normalize();
    let u_displace = d.evaluate(&si_eval);

    // Shift si dv in the dv direction
    let mut dv = 0.5 * (si.dvdx.abs() + si.dvdy.abs());
    if dv == 0.0 {
        dv = 0.0005;
    }
    si_eval.p = si.p + dv * si.shading.dpdv;
    si_eval.uv = si.uv + Vector2f::new(dv, 0.0);
    si_eval.n = (si.shading.dpdv.cross(&si.shading.dpdv) + dv * si.dndv).normalize();
    let v_displace = d.evaluate(&si_eval);

    let displace = d.evaluate(si);

    // Compute bump-mapped differential geometry
    let dpdu = si.shading.dpdu + (u_displace - displace) / du * si.shading.n +
               displace * si.shading.dndu;
    let dpdv = si.shading.dpdv + (v_displace - displace) / dv * si.shading.n +
               displace * si.shading.dndv;
    let dndu = si.shading.dndu;
    let dndv = si.shading.dndv;
    si.set_shading_geometry(&dpdu, &dpdv, &dndu, &dndv, false);
}
