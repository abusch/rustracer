use geometry::Geometry;
use ray::Ray;
use intersection::Intersection;
use material::Material;
use std::rc::Rc;

pub struct Instance {
    pub geom: Rc<Geometry>,
    pub material: Material,
}

impl Instance {
    pub fn new(g: Rc<Geometry>, material: Material) -> Instance {
        Instance {geom: g, material: material}
    }

    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        self.geom.intersect(ray).map(|dg| Intersection::new(dg, self))
    }
}
