use Transform;
use geometry::Geometry;
use ray::Ray;
use intersection::Intersection;
use material::Material;
use na::Inverse;

pub struct Instance {
    pub geom: Box<Geometry + Sync + Send>,
    pub material: Material,
    pub transform: Transform,
    pub transform_inv: Transform,
}

impl Instance {
    pub fn new(g: Box<Geometry + Sync + Send>,
               material: Material,
               transform: Transform)
               -> Instance {
        Instance {
            geom: g,
            material: material,
            transform: transform,
            transform_inv: transform.inverse().unwrap(),
        }
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        let mut local = self.transform_inv * *ray;
        self.geom.intersect(&mut local).map(|mut dg| {
            ray.t_max = local.t_max;
            dg.phit = self.transform * dg.phit;
            dg.nhit = self.transform * dg.nhit;
            Intersection::new(dg, self)
        })
    }
}
