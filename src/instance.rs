use std::sync::Arc;

use Point;
use Transform;
use geometry::BoundedGeometry;
use geometry::{BBox, Bounded};
use ray::Ray;
use intersection::Intersection;
use material::Material;
use na::Inverse;

pub struct Instance {
    pub geom: Box<BoundedGeometry + Sync + Send>,
    pub material: Arc<Material + Sync + Send>,
    pub transform: Transform,
    pub transform_inv: Transform,
    bounds: BBox,
}

impl Instance {
    pub fn new(g: Box<BoundedGeometry + Sync + Send>,
               material: Arc<Material + Send + Sync>,
               transform: Transform)
               -> Instance {

        let b = g.get_world_bounds();
        let mut bbox = BBox::new();
        bbox.extend(transform * Point::new(b.bounds[0].x, b.bounds[0].y, b.bounds[0].z));
        bbox.extend(transform * Point::new(b.bounds[1].x, b.bounds[0].y, b.bounds[0].z));
        bbox.extend(transform * Point::new(b.bounds[0].x, b.bounds[1].y, b.bounds[0].z));
        bbox.extend(transform * Point::new(b.bounds[0].x, b.bounds[0].y, b.bounds[1].z));
        bbox.extend(transform * Point::new(b.bounds[1].x, b.bounds[1].y, b.bounds[0].z));
        bbox.extend(transform * Point::new(b.bounds[1].x, b.bounds[0].y, b.bounds[1].z));
        bbox.extend(transform * Point::new(b.bounds[0].x, b.bounds[1].y, b.bounds[1].z));
        bbox.extend(transform * Point::new(b.bounds[1].x, b.bounds[1].y, b.bounds[1].z));

        Instance {
            geom: g,
            material: material.clone(),
            transform: transform,
            transform_inv: transform.inverse().unwrap(),
            bounds: bbox,
        }
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        let mut local = self.transform_inv * *ray;
        self.geom.intersect(&mut local).map(|mut dg| {
            ray.t_max = local.t_max;
            dg.transform(self.transform, self.transform_inv);
            Intersection::new(dg, -ray.d, self.material.clone())
        })
    }

    pub fn intersect_p(&self, ray: &mut Ray) -> bool {
        let mut local = self.transform_inv * *ray;
        self.geom.intersect_p(&mut local)
    }
}

impl Bounded for Instance {
    fn get_world_bounds(&self) -> BBox {
        self.bounds
    }
}
