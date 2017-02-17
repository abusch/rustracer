use std::cmp::min;
use std::mem::replace;
use std::sync::Arc;
use std::path::Path;

use it;
use na::Point3;

use {Point3f, Vector3f, Transform};
use bounds::{Axis, Bounds3f};
use interaction::SurfaceInteraction;
use light::AreaLight;
use material::{Material, TransportMode};
use primitive::{Primitive, GeometricPrimitive};
use ray::Ray;
use shapes::mesh;
use shapes::mesh::Triangle;

pub struct BVH<T> {
    max_prims_per_node: usize,
    primitives: Vec<T>,
    nodes: Vec<LinearBVHNode>,
}

impl<T: Primitive> BVH<T> {
    pub fn from_mesh_file(file: &Path,
                          model: &str,
                          material: Arc<Material + Send + Sync>,
                          transform: &Transform)
                          -> BVH<GeometricPrimitive> {
        let mut triangles: Vec<Triangle> = mesh::load_triangle_mesh(file, model, transform);
        let mut prims = triangles.drain(..)
            .map(|t| {
                GeometricPrimitive {
                    shape: Arc::new(t),
                    area_light: None,
                    material: Some(material.clone()),
                }
            })
            .collect();

        BVH::new(4, &mut prims)
    }

    pub fn new(max_prims_per_node: usize, prims: &mut Vec<T>) -> BVH<T> {
        info!("Generating BVH:");

        // 1. Get bounds info
        info!("\tGenerating primitive info");
        let mut primitive_info: Vec<BVHPrimitiveInfo> = prims.iter()
            .enumerate()
            .map(|(i, p)| BVHPrimitiveInfo::new(i, p.world_bounds()))
            .collect();

        // 2. Build tree
        info!("\tBuilding tree for {} primitives", prims.len());
        let mut total_nodes = 0;
        let mut ordered_prims_idx = Vec::with_capacity(prims.len());
        let root: BVHBuildNode = BVH::<T>::recursive_build(&mut primitive_info,
                                                           0usize,
                                                           prims.len(),
                                                           max_prims_per_node,
                                                           &mut total_nodes,
                                                           &mut ordered_prims_idx);

        // Sort the primitives according to the indices in ordered_prims_idx. This is made tricky
        // due to the fact that a vector owns its elements, which means we can't easily move
        // elements from one vector to another. We have to use drain() instead. Also, zip() and
        // enumerate() are defined on Iterator, and sort_by_key() is defined on Vector, causing a
        // lot of iter()/collect() shennanigans...
        let mut sorted_idx: Vec<(usize, &usize)> = ordered_prims_idx.iter().enumerate().collect();
        sorted_idx.sort_by_key(|i| i.1);

        let mut prims_with_idx: Vec<(T, usize)> =
            prims.drain(..).zip(sorted_idx.iter().map(|i| i.0)).collect();
        prims_with_idx.sort_by_key(|i| i.1);
        let ordered_prims: Vec<T> = prims_with_idx.drain(..).map(|i| i.0).collect();

        info!("\tCreated {} nodes", total_nodes);
        info!("\tOrdered {} primitives", ordered_prims.len());

        // 3. Build flatten representation
        info!("\tFlattening tree");
        let mut nodes = Vec::with_capacity(total_nodes);
        BVH::<T>::flatten_bvh(&root, &mut nodes);
        assert!(nodes.len() == total_nodes);

        BVH {
            max_prims_per_node: min(max_prims_per_node, 255),
            primitives: ordered_prims,
            nodes: nodes,
        }
    }

    fn recursive_build(build_data: &mut Vec<BVHPrimitiveInfo>,
                       start: usize,
                       end: usize,
                       max_prims_per_node: usize,
                       total_nodes: &mut usize,
                       ordered_prims: &mut Vec<usize>)
                       -> BVHBuildNode {
        *total_nodes += 1;
        let n_primitives = end - start;
        assert!(start != end);
        // Compute bounds of all primitives in node
        let bbox = build_data[start..end]
            .iter()
            .fold(Bounds3f::new(), |b, pi| Bounds3f::union(&b, &pi.bounds));
        if n_primitives == 1 {
            // Create leaf
            let first_prim_offset = ordered_prims.len();
            for pi in build_data[start..end].iter() {
                ordered_prims.push(pi.prim_number);
            }
            BVHBuildNode::leaf(first_prim_offset, n_primitives, bbox)
        } else {
            // Compute bounds of primitive centroids
            let centroids_bounds = build_data[start..end]
                .iter()
                .fold(Bounds3f::new(),
                      |bb, pi| Bounds3f::union_point(&bb, &pi.centroid));
            // Choose split dimension
            let dimension = centroids_bounds.maximum_extent();
            // Partition primitives into 2 sets and build children
            if centroids_bounds[0][dimension] == centroids_bounds[1][dimension] {
                let first_prim_offset = ordered_prims.len();
                for pi in build_data[start..end].iter() {
                    ordered_prims.push(pi.prim_number);
                }
                return BVHBuildNode::leaf(first_prim_offset, n_primitives, bbox);
            }
            // Partition primitives based on split method (here split middle)
            let pmid = 0.5 * (centroids_bounds[0][dimension] + centroids_bounds[1][dimension]);
            let mut mid = it::partition(build_data[start..end].iter_mut(),
                                        |pi| pi.centroid[dimension] < pmid) +
                          start;
            if mid == start || mid == end {
                // If partition failed, used Split Equal method
                build_data[start..end].sort_by(|p1, p2| {
                    p1.centroid[dimension]
                        .partial_cmp(&p2.centroid[dimension])
                        .unwrap()
                });
                mid = (start + end) / 2;
            }

            BVHBuildNode::interior(dimension,
                                   Box::new(BVH::<T>::recursive_build(build_data,
                                                                      start,
                                                                      mid,
                                                                      max_prims_per_node,
                                                                      total_nodes,
                                                                      ordered_prims)),
                                   Box::new(BVH::<T>::recursive_build(build_data,
                                                                      mid,
                                                                      end,
                                                                      max_prims_per_node,
                                                                      total_nodes,
                                                                      ordered_prims)))


        }
    }

    fn flatten_bvh(node: &BVHBuildNode, nodes: &mut Vec<LinearBVHNode>) -> usize {
        let offset = nodes.len();

        match *node {
            BVHBuildNode::Leaf { first_prim_offset, num_prims, .. } => {
                let linear_node = LinearBVHNode {
                    bounds: *node.bounds(),
                    data: LinearBVHNodeData::Leaf {
                        num_prims: num_prims,
                        primitives_offset: first_prim_offset,
                    },
                };
                nodes.push(linear_node);
            }
            BVHBuildNode::Interior { split_axis, ref children, .. } => {
                let linear_node = LinearBVHNode {
                    bounds: *node.bounds(),
                    data: LinearBVHNodeData::Interior {
                        axis: split_axis,
                        second_child_offset: 0,
                    },
                };
                nodes.push(linear_node);
                BVH::<T>::flatten_bvh(&*children[0], nodes);
                let second_offset = BVH::<T>::flatten_bvh(&*children[1], nodes);
                replace(&mut nodes[offset].data,
                        LinearBVHNodeData::Interior {
                            axis: split_axis,
                            second_child_offset: second_offset,
                        });
            }
        }

        offset
    }
}

impl<T: Primitive> Primitive for BVH<T> {
    fn world_bounds(&self) -> Bounds3f {
        self.primitives[0].world_bounds()
    }

    fn intersect(&self, ray: &mut Ray) -> Option<SurfaceInteraction> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut result = None;

        let mut to_visit_offset = 0;
        let mut current_node_idx = 0;
        let mut nodes_to_visit = [0; 64];
        let inv_dir = Vector3f::new(1.0 / ray.d.x, 1.0 / ray.d.y, 1.0 / ray.d.z);
        let dir_is_neg =
            [(inv_dir.x < 0.0) as usize, (inv_dir.y < 0.0) as usize, (inv_dir.z < 0.0) as usize];
        loop {
            let linear_node = &self.nodes[current_node_idx];
            if linear_node.bounds.intersect_p_fast(ray, &inv_dir, &dir_is_neg) {
                match linear_node.data {
                    LinearBVHNodeData::Leaf { num_prims, primitives_offset } => {
                        for i in 0..num_prims {
                            result =
                                self.primitives[primitives_offset + i].intersect(ray).or(result);
                        }
                        if to_visit_offset == 0 {
                            break;
                        }
                        to_visit_offset -= 1;
                        current_node_idx = nodes_to_visit[to_visit_offset];
                    }
                    LinearBVHNodeData::Interior { axis, second_child_offset, .. } => {
                        let axis_num = match axis {
                            Axis::X => 0,
                            Axis::Y => 1,
                            Axis::Z => 2,
                        };
                        if dir_is_neg[axis_num] != 0 {
                            nodes_to_visit[to_visit_offset] = current_node_idx + 1;
                            to_visit_offset += 1;
                            current_node_idx = second_child_offset;
                        } else {
                            nodes_to_visit[to_visit_offset] = second_child_offset;
                            to_visit_offset += 1;
                            current_node_idx += 1;
                        }
                    }
                }
            } else {
                if to_visit_offset == 0 {
                    break;
                }
                to_visit_offset -= 1;
                current_node_idx = nodes_to_visit[to_visit_offset];
            }

        }
        result
    }

    fn intersect_p(&self, ray: &Ray) -> bool {
        if self.nodes.is_empty() {
            return false;
        }

        let mut to_visit_offset = 0;
        let mut current_node_idx = 0;
        let mut nodes_to_visit = [0; 64];
        let inv_dir = Vector3f::new(1.0 / ray.d.x, 1.0 / ray.d.y, 1.0 / ray.d.z);
        let dir_is_neg =
            [(inv_dir.x < 0.0) as usize, (inv_dir.y < 0.0) as usize, (inv_dir.z < 0.0) as usize];
        loop {
            let linear_node = &self.nodes[current_node_idx];
            if linear_node.bounds.intersect_p_fast(ray, &inv_dir, &dir_is_neg) {
                match linear_node.data {
                    LinearBVHNodeData::Leaf { num_prims, primitives_offset } => {
                        for i in 0..num_prims {

                            if self.primitives[primitives_offset + i].intersect_p(ray) {
                                return true;
                            }
                        }
                        if to_visit_offset == 0 {
                            break;
                        }
                        to_visit_offset -= 1;
                        current_node_idx = nodes_to_visit[to_visit_offset];
                    }
                    LinearBVHNodeData::Interior { axis, second_child_offset, .. } => {
                        let axis_num = match axis {
                            Axis::X => 0,
                            Axis::Y => 1,
                            Axis::Z => 2,
                        };
                        if dir_is_neg[axis_num] != 0 {
                            nodes_to_visit[to_visit_offset] = current_node_idx + 1;
                            to_visit_offset += 1;
                            current_node_idx = second_child_offset;
                        } else {
                            nodes_to_visit[to_visit_offset] = second_child_offset;
                            to_visit_offset += 1;
                            current_node_idx += 1;
                        }
                    }
                }
            } else {
                if to_visit_offset == 0 {
                    break;
                }
                to_visit_offset -= 1;
                current_node_idx = nodes_to_visit[to_visit_offset];
            }

        }
        false
    }

    fn area_light(&self) -> Option<Arc<AreaLight + Send + Sync>> {
        panic!("area_light() should not be called on an Aggregate Primitive!");
    }

    fn material(&self) -> Option<Arc<Material + Send + Sync>> {
        panic!("material() should not be called on an Aggregate Primitive!");
    }

    fn compute_scattering_functions(&self,
                                    isect: &mut SurfaceInteraction,
                                    mode: TransportMode,
                                    allow_multiple_lobes: bool) {
        panic!("compute_scattering_functions() should not be called on an Aggregate Primitive!");
    }
}

struct BVHPrimitiveInfo {
    pub prim_number: usize,
    pub centroid: Point3f,
    pub bounds: Bounds3f,
}

impl BVHPrimitiveInfo {
    fn new(pn: usize, bb: Bounds3f) -> BVHPrimitiveInfo {
        BVHPrimitiveInfo {
            prim_number: pn,
            centroid: Point3::from_coordinates(0.5 * bb[0].coords + 0.5 * bb[1].coords),
            bounds: bb,
        }
    }
}

enum BVHBuildNode {
    Interior {
        bounds: Bounds3f,
        children: [Box<BVHBuildNode>; 2],
        split_axis: Axis,
    },
    Leaf {
        bounds: Bounds3f,
        first_prim_offset: usize,
        num_prims: usize,
    },
}

impl BVHBuildNode {
    fn interior(axis: Axis, child1: Box<BVHBuildNode>, child2: Box<BVHBuildNode>) -> BVHBuildNode {
        let bbox = Bounds3f::union(child1.bounds(), child2.bounds());
        BVHBuildNode::Interior {
            bounds: bbox,
            children: [child1, child2],
            split_axis: axis,
        }
    }

    fn leaf(first_prim_offset: usize, num_prims: usize, bbox: Bounds3f) -> BVHBuildNode {
        BVHBuildNode::Leaf {
            bounds: bbox,
            first_prim_offset: first_prim_offset,
            num_prims: num_prims,
        }
    }

    fn bounds(&self) -> &Bounds3f {
        match *self {
            BVHBuildNode::Interior { ref bounds, .. } |
            BVHBuildNode::Leaf { ref bounds, .. } => bounds,
        }
    }
}

#[derive(Debug)]
enum LinearBVHNodeData {
    Interior {
        second_child_offset: usize,
        axis: Axis,
    },
    Leaf {
        primitives_offset: usize,
        num_prims: usize,
    },
}

#[derive(Debug)]
struct LinearBVHNode {
    bounds: Bounds3f,
    data: LinearBVHNodeData,
}
