use std::sync::Arc;
use std::cmp::{min, Ordering};

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
        println!("Root node bounding box: {:?}", root.bounds());
        let mut ordered_primitives = Vec::with_capacity(prims.len());
        for i in ordered_prims {
            ordered_primitives.push(prims[i].clone());
        }

        println!("Created {} nodes", total_nodes);

        // 3. Build flatten representation
        println!("Flattening tree");
        let mut nodes = Vec::with_capacity(total_nodes);
        BVH::<T>::flatten_bvh(&root, &mut nodes);

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
        // println!("recursive_build() for range ({}, {}), total_nodes={}",
        //          start,
        //          end,
        //          total_nodes);
        *total_nodes += 1;
        let n_primitives = end - start;
        let first_prim_offset = ordered_prims.len();
        // Compute bounds of all primitives in node
        let bbox = build_data.iter()
            .fold(BBox::new(), |b, pi| BBox::union(&b, &pi.bounds));
        // println!("bbox = {:?}", bbox);
        if n_primitives <= max_prims_per_node {
            // println!("Creating leaf");
            // Create leaf
            for pi in build_data[start..end].iter() {
                ordered_prims.push(pi.prim_number);
            }
            return BVHBuildNode::leaf(first_prim_offset, n_primitives, bbox);
        } else {
            // Compute bounds of primitive centroids
            let centroids_bounds = build_data[start..end]
                .iter()
                .fold(BBox::new(), |bb, pi| BBox::union_point(&bb, &pi.centroid));
            // println!("centroid bounds: {:?}", centroids_bounds);
            // Choose split dimension
            let dimension = centroids_bounds.maximum_extent();
            // Partition primitives into 2 sets and build children
            // let mid = (start + end) / 2;
            if centroids_bounds.bounds[0] == centroids_bounds.bounds[1] {
                return BVHBuildNode::leaf(first_prim_offset, n_primitives, bbox);
            }
            // Partition primitives based on split method
            let pmid = 0.5 *
                       (centroids_bounds.bounds[0][dimension] +
                        centroids_bounds.bounds[1][dimension]);
            let mut mid = partition::partition(build_data[start..end].iter_mut(),
                                               |pi| pi.centroid[dimension] < pmid) +
                          start;
            if mid == start || mid == end {
                build_data[start..end].sort_by(|p1, p2| {
                    p1.centroid[dimension]
                        .partial_cmp(&p2.centroid[dimension])
                        .unwrap()
                });
                mid = (start + end) / 2;
            }
            // println!("mid={}", mid);

            return BVHBuildNode::interior(dimension,
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
                                                                             ordered_prims)));


        }
    }

    fn flatten_bvh(node: &BVHBuildNode, nodes: &mut Vec<LinearBVHNode>) -> usize {
        let offset = nodes.len();

        match node {
            &BVHBuildNode::Leaf { first_prim_offset, num_prims, .. } => {
                let linear_node = LinearBVHNode {
                    bounds: *node.bounds(),
                    data: LinearBVHNodeData::Leaf {
                        num_prims: num_prims,
                        primitives_offset: first_prim_offset,
                    },
                };
                nodes.push(linear_node);
            }
            &BVHBuildNode::Interior { split_axis, ref children, .. } => {
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
                nodes[offset].data = LinearBVHNodeData::Interior {
                    axis: split_axis,
                    second_child_offset: second_offset,
                };
            }
        }

        return offset;
    }
}

impl<T: Bounded + Geometry> Geometry for BVH<T> {
    fn intersect(&self, ray: &mut Ray) -> Option<DifferentialGeometry> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut result = None;
        // let origin = ray.at(ray.t_min);

        let mut todo_offset = 0;
        let mut node_num = 0;
        let mut todo = [0; 64];
        let inv_dir = Vector::new(1.0 / ray.dir.x, 1.0 / ray.dir.y, 1.0 / ray.dir.z);
        let dir_is_neg =
            [(inv_dir.x < 0.0) as usize, (inv_dir.y < 0.0) as usize, (inv_dir.z < 0.0) as usize];
        loop {
            let ref linear_node = self.nodes[node_num];
            if linear_node.bounds.intersect_p(ray, &inv_dir, &dir_is_neg) {
                match linear_node.data {
                    LinearBVHNodeData::Interior { axis, second_child_offset, .. } => {
                        let axis_num = match axis {
                            Axis::X => 0,
                            Axis::Y => 1,
                            Axis::Z => 2,
                        };
                        if dir_is_neg[axis_num] == 1 {
                            todo[todo_offset] = node_num + 1;
                            todo_offset += 1;
                            node_num = second_child_offset;
                        } else {
                            todo[todo_offset] = second_child_offset;
                            todo_offset += 1;
                            node_num += 1;
                        }
                    }
                    LinearBVHNodeData::Leaf { num_prims, primitives_offset } => {
                        for i in 0..num_prims {
                            if let Some(isect) = self.primitives[primitives_offset + i]
                                .intersect(ray) {
                                return Some(isect);
                            }
                        }
                        if todo_offset == 0 {
                            break;
                        }
                        todo_offset -= 1;
                        node_num = todo[todo_offset];
                    }
                }
            } else {
                if todo_offset == 0 {
                    break;
                }
                todo_offset -= 1;
                node_num = todo[todo_offset];
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
        BVHBuildNode::Interior {
            bounds: BBox::union(&child1.bounds(), &child2.bounds()),
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
            BVHBuildNode::Interior { ref bounds, .. } => bounds,
            BVHBuildNode::Leaf { ref bounds, .. } => bounds,
        }
    }
}

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

struct LinearBVHNode {
    bounds: BBox,
    data: LinearBVHNodeData,
}
