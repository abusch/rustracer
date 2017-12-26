use std::cmp::min;
use std::mem::replace;
use std::sync::Arc;

use it;
use light_arena::Allocator;

use {Point3f, Vector3f};
use bounds::{Axis, Bounds3f};
use interaction::SurfaceInteraction;
use light::AreaLight;
use material::{Material, TransportMode};
use paramset::ParamSet;
use primitive::{GeometricPrimitive, Primitive};
use ray::Ray;
use shapes::Shape;

stat_memory_counter!("Memory/BVH tree", tree_bytes);
stat_ratio!("BVH/Primitives per leaf node", total_primitives_per_leaf);
stat_counter!("BVH/Interior nodes", interior_nodes);
stat_counter!("BVH/Leaf nodes", leaf_nodes);
pub fn init_stats() {
    tree_bytes::init();
    total_primitives_per_leaf::init();
    interior_nodes::init();
    leaf_nodes::init();
}

#[derive(Copy, Clone, Debug)]
pub enum SplitMethod {
    Middle,
    EqualCounts,
    SAH,
}

#[derive(Debug)]
pub struct BVH {
    max_prims_per_node: usize,
    primitives: Vec<Arc<Primitive + Send + Sync>>,
    nodes: Vec<LinearBVHNode>,
}

impl BVH {
    pub fn from_triangles(mut tris: Vec<Arc<Shape + Send + Sync>>,
                          material: &Arc<Material + Send + Sync>)
                          -> BVH {
        let mut prims: Vec<Arc<Primitive + Send + Sync>> = tris.drain(..)
            .map(|t| {
                let prim = GeometricPrimitive {
                    shape: Arc::clone(&t),
                    area_light: None,
                    material: Some(Arc::clone(material)),
                };
                let b: Arc<Primitive + Send + Sync> = Arc::new(prim);
                b
            })
            .collect();

        BVH::new(1, &mut prims, SplitMethod::SAH)
    }

    pub fn create(prims: &mut Vec<Arc<Primitive + Send + Sync>>, ps: &mut ParamSet) -> BVH {
        let split_method_name = ps.find_one_string("splitmethod", "sah".into());
        let split_method = if split_method_name == "sah" {
            SplitMethod::SAH
        } else if split_method_name == "middle" {
            SplitMethod::Middle
        } else {
            warn!("Unknown (or unimplemented) BVH split method {}.  Using \"sah\"",
                  split_method_name);
            SplitMethod::SAH
        };
        let max_prims_per_node = ps.find_one_int("maxnodeprims", 4);
        BVH::new(max_prims_per_node as usize, prims, split_method)
    }

    pub fn new(max_prims_per_node: usize,
               prims: &mut Vec<Arc<Primitive + Send + Sync>>,
               split_method: SplitMethod)
               -> BVH {
        info!("Generating BVH with method {:?}:", split_method);

        // 1. Get bounds info
        info!("\tGenerating primitive info");
        let mut primitive_info: Vec<BVHPrimitiveInfo> = prims
            .iter()
            .enumerate()
            .map(|(i, p)| BVHPrimitiveInfo::new(i, p.world_bounds()))
            .collect();

        // 2. Build tree
        info!("\tBuilding tree for {} primitives", prims.len());
        let mut total_nodes = 0;
        let mut ordered_prims = Vec::with_capacity(prims.len());
        let root: BVHBuildNode = BVH::recursive_build(prims,
                                                      &mut primitive_info,
                                                      0usize,
                                                      prims.len(),
                                                      max_prims_per_node,
                                                      &mut total_nodes,
                                                      &mut ordered_prims,
                                                      split_method);

        info!("\tCreated {} nodes", total_nodes);

        // 3. Build flatten representation
        info!("\tFlattening tree");
        let mut nodes = Vec::with_capacity(total_nodes);
        BVH::flatten_bvh(&root, &mut nodes);
        assert_eq!(nodes.len(), total_nodes);


        let bvh = BVH {
            max_prims_per_node: min(max_prims_per_node, 255),
            primitives: ordered_prims,
            nodes: nodes,
        };
        tree_bytes::add((total_nodes * ::std::mem::size_of::<LinearBVHNode>() +
                         ::std::mem::size_of_val(&bvh) +
                         prims.len() * ::std::mem::size_of_val(&prims[0])) as
                        u64);
        info!("BVH created with {} nodes for {} primitives",
              total_nodes,
              bvh.primitives.len());

        bvh
    }

    fn recursive_build(primitives: &[Arc<Primitive + Send + Sync>],
                       primitive_info: &mut Vec<BVHPrimitiveInfo>,
                       start: usize,
                       end: usize,
                       max_prims_per_node: usize,
                       total_nodes: &mut usize,
                       ordered_prims: &mut Vec<Arc<Primitive + Send + Sync>>,
                       split_method: SplitMethod)
                       -> BVHBuildNode {
        *total_nodes += 1;
        let n_primitives = end - start;
        assert_ne!(start, end);
        // Compute bounds of all primitives in node
        let bounds = primitive_info[start..end]
            .iter()
            .fold(Bounds3f::new(), |b, pi| Bounds3f::union(&b, &pi.bounds));
        if n_primitives == 1 {
            // Create leaf
            let first_prim_offset = ordered_prims.len();
            for pi in primitive_info[start..end].iter() {
                let prim_num = pi.prim_number;
                ordered_prims.push(Arc::clone(&primitives[prim_num]));
            }
            BVHBuildNode::leaf(first_prim_offset, n_primitives, bounds)
        } else {
            // Compute bounds of primitive centroids
            let centroids_bounds = primitive_info[start..end]
                .iter()
                .fold(Bounds3f::new(),
                      |bb, pi| Bounds3f::union_point(&bb, &pi.centroid));
            // Choose split dimension
            let dimension = centroids_bounds.maximum_extent();
            // Partition primitives into 2 sets and build children
            if centroids_bounds[0][dimension] == centroids_bounds[1][dimension] {
                let first_prim_offset = ordered_prims.len();
                for pi in primitive_info[start..end].iter() {
                    let prim_num = pi.prim_number;
                    ordered_prims.push(Arc::clone(&primitives[prim_num]));
                }
                return BVHBuildNode::leaf(first_prim_offset, n_primitives, bounds);
            }
            // Partition primitives based on split method (here split middle)
            let mut mid;
            match split_method {
                SplitMethod::Middle => {
                    let pmid = 0.5 *
                               (centroids_bounds[0][dimension] + centroids_bounds[1][dimension]);
                    mid = start +
                          it::partition(primitive_info[start..end].iter_mut(),
                                        |pi| pi.centroid[dimension] < pmid) +
                          start;
                    if mid == start || mid == end {
                        // If partition failed, used Split Equal method
                        primitive_info[start..end].sort_by(|p1, p2| {
                                                               p1.centroid[dimension]
                                                                   .partial_cmp(&p2.centroid
                                                                                     [dimension])
                                                                   .unwrap()
                                                           });
                        mid = (start + end) / 2;
                    }
                }
                SplitMethod::EqualCounts => unimplemented!(),
                SplitMethod::SAH => {
                    // Partition primitives using approximate SAH
                    if n_primitives <= 2 {
                        // Partition primitives into equally-sized subsets
                        mid = (start + end) / 2;
                        if start != end - 1 &&
                           primitive_info[end - 1].centroid[dimension] <
                           primitive_info[start].centroid[dimension] {
                            primitive_info.swap(start, end - 1);
                        }
                    } else {
                        const N_BUCKETS: usize = 12;
                        // Allocate `BucketInfo for SAH partition buckets
                        let mut buckets = [BucketInfo::default(); 12];

                        // Initialize `BucketInfo` for SAH partition buckets
                        for i in start..end {
                            let mut b = (N_BUCKETS as f32 *
                                         centroids_bounds.offset(&primitive_info[i].centroid)
                                             [dimension]) as
                                        usize;
                            if b == N_BUCKETS {
                                b = N_BUCKETS - 1;
                            }
                            assert!(b < N_BUCKETS);
                            buckets[b].count += 1;
                            buckets[b].bounds = Bounds3f::union(&buckets[b].bounds,
                                                                &primitive_info[i].bounds);
                        }

                        // Compute costs for splitting after each bucket
                        let mut cost = [0.0; N_BUCKETS - 1];
                        for i in 0..(N_BUCKETS - 1) {
                            let mut b0 = Bounds3f::new();
                            let mut b1 = Bounds3f::new();
                            let mut count0 = 0;
                            let mut count1 = 0;
                            for j in 0..(i + 1) {
                                b0 = Bounds3f::union(&b0, &buckets[j].bounds);
                                count0 += buckets[j].count;
                            }
                            for j in (i + 1)..N_BUCKETS {
                                b1 = Bounds3f::union(&b1, &buckets[j].bounds);
                                count1 += buckets[j].count;
                            }
                            cost[i] = 1.0 +
                                      (count0 as f32 * b0.surface_area() +
                                       count1 as f32 * b1.surface_area()) /
                                      bounds.surface_area();
                        }

                        // Find bucket to split at that minimizes SAH metric
                        let mut min_cost = cost[0];
                        let mut min_cost_split_bucket = 0;
                        for i in 1..(N_BUCKETS - 1) {
                            if cost[i] < min_cost {
                                min_cost = cost[i];
                                min_cost_split_bucket = i;
                            }
                        }

                        // Either create leaf of split primitives at selected SAH bucket
                        let leaf_cost = n_primitives as f32;
                        if n_primitives > max_prims_per_node || min_cost < leaf_cost {
                            mid = start +
                                  it::partition(primitive_info[start..end].iter_mut(), |pi| {
                                let mut b = (N_BUCKETS as f32 *
                                             centroids_bounds.offset(&pi.centroid)[dimension]) as
                                            usize;
                                if b == N_BUCKETS {
                                    b = N_BUCKETS - 1;
                                }
                                assert!(b < N_BUCKETS);
                                b <= min_cost_split_bucket
                            });
                        } else {
                            // Create leaf `BVHBuildNode`
                            let first_prim_offset = ordered_prims.len();
                            for i in start..end {
                                let prim_num = primitive_info[i].prim_number;
                                ordered_prims.push(Arc::clone(&primitives[prim_num]));
                            }
                            return BVHBuildNode::leaf(first_prim_offset, n_primitives, bounds);
                        }
                    }
                }
            }

            let right = Box::new(BVH::recursive_build(primitives,
                                                      primitive_info,
                                                      mid,
                                                      end,
                                                      max_prims_per_node,
                                                      total_nodes,
                                                      ordered_prims,
                                                      split_method));
            let left = Box::new(BVH::recursive_build(primitives,
                                                     primitive_info,
                                                     start,
                                                     mid,
                                                     max_prims_per_node,
                                                     total_nodes,
                                                     ordered_prims,
                                                     split_method));
            BVHBuildNode::interior(dimension, left, right)
        }
    }

    fn flatten_bvh(node: &BVHBuildNode, nodes: &mut Vec<LinearBVHNode>) -> usize {
        let offset = nodes.len();

        match *node {
            BVHBuildNode::Leaf {
                first_prim_offset,
                num_prims,
                ..
            } => {
                let linear_node = LinearBVHNode {
                    bounds: *node.bounds(),
                    data: LinearBVHNodeData::Leaf {
                        num_prims: num_prims,
                        primitives_offset: first_prim_offset,
                    },
                };
                nodes.push(linear_node);
            }
            BVHBuildNode::Interior {
                split_axis,
                ref children,
                ..
            } => {
                let linear_node = LinearBVHNode {
                    bounds: *node.bounds(),
                    data: LinearBVHNodeData::Interior {
                        axis: split_axis,
                        second_child_offset: 0,
                    },
                };
                nodes.push(linear_node);
                BVH::flatten_bvh(&*children[0], nodes);
                let second_offset = BVH::flatten_bvh(&*children[1], nodes);
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

impl Primitive for BVH {
    fn world_bounds(&self) -> Bounds3f {
        self.nodes[0].bounds
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
        let dir_is_neg = [(inv_dir.x < 0.0) as usize,
                          (inv_dir.y < 0.0) as usize,
                          (inv_dir.z < 0.0) as usize];
        loop {
            let linear_node = &self.nodes[current_node_idx];
            if linear_node
                   .bounds
                   .intersect_p_fast(ray, &inv_dir, &dir_is_neg) {
                match linear_node.data {
                    LinearBVHNodeData::Leaf {
                        num_prims,
                        primitives_offset,
                    } => {
                        for i in 0..num_prims {
                            result = self.primitives[primitives_offset + i]
                                .intersect(ray)
                                .or(result);
                        }
                        if to_visit_offset == 0 {
                            break;
                        }
                        to_visit_offset -= 1;
                        current_node_idx = nodes_to_visit[to_visit_offset];
                    }
                    LinearBVHNodeData::Interior {
                        axis,
                        second_child_offset,
                        ..
                    } => {
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
        let dir_is_neg = [(inv_dir.x < 0.0) as usize,
                          (inv_dir.y < 0.0) as usize,
                          (inv_dir.z < 0.0) as usize];
        loop {
            let linear_node = &self.nodes[current_node_idx];
            if linear_node
                   .bounds
                   .intersect_p_fast(ray, &inv_dir, &dir_is_neg) {
                match linear_node.data {
                    LinearBVHNodeData::Leaf {
                        num_prims,
                        primitives_offset,
                    } => {
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
                    LinearBVHNodeData::Interior {
                        axis,
                        second_child_offset,
                        ..
                    } => {
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

    fn compute_scattering_functions<'a, 'b>(&self,
                                            _isect: &mut SurfaceInteraction<'a, 'b>,
                                            _mode: TransportMode,
                                            _allow_multiple_lobes: bool,
                                            _arena: &'b Allocator) {
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
            centroid: 0.5 * bb[0] + 0.5 * bb[1],
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
        interior_nodes::inc();
        BVHBuildNode::Interior {
            bounds: bbox,
            children: [child1, child2],
            split_axis: axis,
        }
    }

    fn leaf(first_prim_offset: usize, num_prims: usize, bbox: Bounds3f) -> BVHBuildNode {
        leaf_nodes::inc();
        total_primitives_per_leaf::add(num_prims as u64);
        total_primitives_per_leaf::inc_total();
        BVHBuildNode::Leaf {
            bounds: bbox,
            first_prim_offset: first_prim_offset,
            num_prims: num_prims,
        }
    }

    fn bounds(&self) -> &Bounds3f {
        match self {
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
    pub bounds: Bounds3f,
    data: LinearBVHNodeData,
}

#[derive(Debug, Default, Copy, Clone)]
struct BucketInfo {
    pub count: usize,
    pub bounds: Bounds3f,
}
