use std::sync::Arc;
use std::cmp::min;
use std::mem::replace;

use ::{Point, Vector};
use geometry::{Axis, Geometry, BBox, Bounded, DifferentialGeometry};
use partition;
use ray::Ray;

pub struct BVH<T: Bounded + Geometry> {
    max_prims_per_node: usize,
    primitives: Vec<Arc<T>>,
    nodes: Vec<LinearBVHNode>,
}

impl<T: Bounded + Geometry> BVH<T> {
    pub fn new(max_prims_per_node: usize, prims: Vec<Arc<T>>) -> BVH<T> {
        println!("Generating BVH...");

        // 1. Get bounds info
        println!("\tGenerating primitive info...");
        let mut build_data: Vec<BVHPrimitiveInfo> = prims.iter()
            .enumerate()
            .map(|(i, p)| BVHPrimitiveInfo::new(i, p.get_world_bounds()))
            .collect();

        // 2. Build tree
        println!("\tBuilding tree...");
        let mut total_nodes = 0;
        let mut ordered_prims = Vec::with_capacity(prims.len());
        let root: BVHBuildNode = BVH::<T>::recursive_build(&mut build_data,
                                                           0usize,
                                                           prims.len(),
                                                           max_prims_per_node,
                                                           &mut total_nodes,
                                                           &mut ordered_prims);
        let mut ordered_primitives = Vec::with_capacity(prims.len());
        for i in ordered_prims {
            ordered_primitives.push(prims[i].clone());
        }
        assert!(prims.len() == ordered_primitives.len());

        println!("\tCreated {} nodes", total_nodes);

        // 3. Build flatten representation
        println!("\tFlattening tree");
        let mut nodes = Vec::with_capacity(total_nodes);
        BVH::<T>::flatten_bvh(&root, &mut nodes);
        assert!(nodes.len() == total_nodes);

        BVH {
            max_prims_per_node: min(max_prims_per_node, 255),
            primitives: ordered_primitives,
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
            .fold(BBox::new(), |b, pi| BBox::union(&b, &pi.bounds));
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
                .fold(BBox::new(), |bb, pi| BBox::union_point(&bb, &pi.centroid));
            // Choose split dimension
            let dimension = centroids_bounds.maximum_extent();
            // Partition primitives into 2 sets and build children
            if centroids_bounds.bounds[0][dimension] == centroids_bounds.bounds[1][dimension] {
                let first_prim_offset = ordered_prims.len();
                for pi in build_data[start..end].iter() {
                    ordered_prims.push(pi.prim_number);
                }
                return BVHBuildNode::leaf(first_prim_offset, n_primitives, bbox);
            }
            // Partition primitives based on split method (here split middle)
            let pmid = 0.5 *
                       (centroids_bounds.bounds[0][dimension] +
                        centroids_bounds.bounds[1][dimension]);
            let mut mid = partition::partition(build_data[start..end].iter_mut(),
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

impl<T: Bounded + Geometry> Geometry for BVH<T> {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut result = None;

        let mut to_visit_offset = 0;
        let mut current_node_idx = 0;
        let mut nodes_to_visit = [0; 64];
        let inv_dir = Vector::new(1.0 / ray.dir.x, 1.0 / ray.dir.y, 1.0 / ray.dir.z);
        let dir_is_neg =
            [(inv_dir.x < 0.0) as usize, (inv_dir.y < 0.0) as usize, (inv_dir.z < 0.0) as usize];
        loop {
            let ref linear_node = self.nodes[current_node_idx];
            if linear_node.bounds.intersect_p(ray, &inv_dir, &dir_is_neg) {
                match linear_node.data {
                    LinearBVHNodeData::Leaf { num_prims, primitives_offset } => {
                        for i in 0..num_prims {
                            if let Some(isect) = self.primitives[primitives_offset + i]
                                .intersect(ray) {
                                result = Some(isect);
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
        result
    }
}

struct BVHPrimitiveInfo {
    pub prim_number: usize,
    pub centroid: Point,
    pub bounds: BBox,
}

impl BVHPrimitiveInfo {
    fn new(pn: usize, bb: BBox) -> BVHPrimitiveInfo {
        BVHPrimitiveInfo {
            prim_number: pn,
            centroid: (0.5 * bb.bounds[0].to_vector() + 0.5 * bb.bounds[1].to_vector()).to_point(),
            bounds: bb,
        }
    }
}

enum BVHBuildNode {
    Interior {
        bounds: BBox,
        children: [Box<BVHBuildNode>; 2],
        split_axis: Axis,
    },
    Leaf {
        bounds: BBox,
        first_prim_offset: usize,
        num_prims: usize,
    },
}

impl BVHBuildNode {
    fn interior(axis: Axis, child1: Box<BVHBuildNode>, child2: Box<BVHBuildNode>) -> BVHBuildNode {
        let bbox = BBox::union(child1.bounds(), child2.bounds());
        BVHBuildNode::Interior {
            bounds: bbox,
            children: [child1, child2],
            split_axis: axis,
        }
    }

    fn leaf(first_prim_offset: usize, num_prims: usize, bbox: BBox) -> BVHBuildNode {
        BVHBuildNode::Leaf {
            bounds: bbox,
            first_prim_offset: first_prim_offset,
            num_prims: num_prims,
        }
    }

    fn bounds(&self) -> &BBox {
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
    bounds: BBox,
    data: LinearBVHNodeData,
}
