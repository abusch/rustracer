use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicPtr, Ordering};

use num::Zero;

use {clamp, Point2f, Point3i, Point3f, Vector3f, Normal3f};
use bounds::Bounds3f;
use interaction::Interaction;
use sampler::lowdiscrepancy::radical_inverse;
use sampling::Distribution1D;
use scene::Scene;

pub trait LightDistribution {
    fn lookup<'a>(&'a self, p: &Point3f) -> &'a Distribution1D;
}

pub struct UniformLightDistribution {
    distrib: Box<Distribution1D>,
}

impl UniformLightDistribution {
    pub fn new(scene: &Scene) -> UniformLightDistribution {
        let prob = vec![1.0; scene.lights.len()];
        UniformLightDistribution { distrib: Box::new(Distribution1D::new(&prob[..])) }
    }
}

impl LightDistribution for UniformLightDistribution {
    fn lookup<'a>(&'a self, _p: &Point3f) -> &'a Distribution1D {
        &self.distrib
    }
}

// SpatialLightDistribution
const INVALID_PACKED_POS: u64 = 0xffffffffffffffff;

pub struct SpatialLightDistribution {
    scene: Arc<Scene>,
    n_voxels: [u32; 3],
    hash_table: Box<[HashEntry]>,
    hash_table_size: usize,
}

impl SpatialLightDistribution {
    pub fn new(scene: Arc<Scene>, max_voxels: u32) -> SpatialLightDistribution {
        // Compute the number of voxels so that the widest scene bounding box
        // dimension has maxVoxels voxels and the other dimensions have a number
        // of voxels so that voxels are roughly cube shaped.

        let b = scene.world_bounds();
        let diag = b.diagonal();
        let b_max = diag[b.maximum_extent()];
        let mut n_voxels = [0; 3];
        for i in 0..3 {
            n_voxels[i] = u32::max(1, f32::round(diag[i] / b_max * max_voxels as f32) as u32);
        }
        let hash_table_size = (4 * n_voxels[0] * n_voxels[1] * n_voxels[2]) as usize;
        let mut hash_table: Vec<HashEntry> = Vec::with_capacity(hash_table_size);
        for i in 0..hash_table_size {
            hash_table.push(HashEntry {
                                packed_pos: AtomicU64::new(INVALID_PACKED_POS),
                                distribution: AtomicPtr::default(),
                            });
        }

        info!("SpatialLightDistribution: scene bounds {}, voxel res {:?}",
              b,
              n_voxels);

        SpatialLightDistribution {
            scene,
            n_voxels,
            hash_table: hash_table.into_boxed_slice(),
            hash_table_size,
        }
    }

    pub fn compute_distribution(&self, pi: &Point3i) -> Distribution1D {
        // Compute the world-space bounding box of the voxel corresponding to
        // |pi|.
        let p0 = Point3f::new(pi[0] as f32 / self.n_voxels[0] as f32,
                              pi[1] as f32 / self.n_voxels[1] as f32,
                              pi[2] as f32 / self.n_voxels[2] as f32);
        let p1 = Point3f::new((pi[0] as f32 + 1.0) / self.n_voxels[0] as f32,
                              (pi[1] as f32 + 1.0) / self.n_voxels[1] as f32,
                              (pi[2] as f32 + 1.0) / self.n_voxels[2] as f32);
        let voxel_bounds = Bounds3f::from_points(&self.scene.world_bounds().lerp(&p0),
                                                 &self.scene.world_bounds().lerp(&p1));

        // Compute the sampling distribution. Sample a number of points inside
        // voxelBounds using a 3D Halton sequence; at each one, sample each
        // light source and compute a weight based on Li/pdf for the light's
        // sample (ignoring visibility between the point in the voxel and the
        // point on the light source) as an approximation to how much the light
        // is likely to contribute to illumination in the voxel.
        let n_samples = 128;
        let mut light_contrib: Vec<f32> = vec![0.0; self.scene.lights.len()];
        for i in 0..n_samples {
            let po = voxel_bounds.lerp(&Point3f::new(radical_inverse(0, i),
                                                     radical_inverse(1, i),
                                                     radical_inverse(2, i)));
            let intr = Interaction::new(po,
                                        Vector3f::zero(),
                                        Vector3f::new(1.0, 0.0, 0.0),
                                        Normal3f::zero());

            // Use the next two Halton dimensions to sample a point on the
            // light source.
            let u = Point2f::new(radical_inverse(3, i), radical_inverse(4, i));
            for j in 0..self.scene.lights.len() {
                let (li, wi, pdf, vis) = self.scene.lights[j].sample_li(&intr, &u);
                if pdf > 0.0 {
                    // TODO: look at tracing shadow rays / computing beam
                    // transmittance.  Probably shouldn't give those full weight
                    // but instead e.g. have an occluded shadow ray scale down
                    // the contribution by 10 or something.
                    light_contrib[j] += li.y() / pdf;
                }
            }
        }

        // We don't want to leave any lights with a zero probability; it's
        // possible that a light contributes to points in the voxel even though
        // we didn't find such a point when sampling above.  Therefore, compute
        // a minimum (small) weight and ensure that all lights are given at
        // least the corresponding probability.
        let sum_contrib: f32 = light_contrib.iter().sum();
        let avg_contrib = sum_contrib / (n_samples * light_contrib.len() as u64) as f32;
        let min_contrib = if avg_contrib > 0.0 {
            0.001 * avg_contrib
        } else {
            1.0
        };
        for i in 0..light_contrib.len() {
            light_contrib[i] = f32::max(light_contrib[i], min_contrib);
        }

        info!("Initialized light distribution in voxel pi={}, avg_contrib={}",
              pi,
              avg_contrib);
        // Compute a sampling distribution from the accumulated contributions.
        Distribution1D::new(&light_contrib[..])
    }
}

impl LightDistribution for SpatialLightDistribution {
    fn lookup<'a>(&'a self, p: &Point3f) -> &'a Distribution1D {
        // First compute integer voxel coordinates for the given point |p|
        // with respect to the overall voxel grid.
        let offset = self.scene.world_bounds().offset(p);
        let mut pi = Point3i::zero();
        for i in 0..3 {
            // The clamp should almost never be necessary, but is there to be
            // robust to computed intersection points being slightly outside
            // the scene bounds due to floating-point roundoff error.
            pi[i] = clamp((offset[i] * self.n_voxels[i] as f32) as i32,
                          0,
                          self.n_voxels[i] as i32 - 1);
        }

        // Pack the 3D integer voxel coordinates into a single 64-bit value.
        let packed_pos: u64 = (pi[0] as u64) << 40 | (pi[1] as u64) << 20 | (pi[2] as u64);
        assert_ne!(packed_pos, INVALID_PACKED_POS);

        // Compute a hash value from the packed voxel coordinates.  We could
        // just take packedPos mod the hash table size, but since packedPos
        // isn't necessarily well distributed on its own, it's worthwhile to do
        // a little work to make sure that its bits values are individually
        // fairly random. For details of and motivation for the following, see:
        // http://zimbry.blogspot.ch/2011/09/better-bit-mixing-improving-on.html
        let mut hash = packed_pos;
        hash ^= hash >> 31;
        hash *= 0x7fb5d329728ea185;
        hash ^= hash >> 27;
        hash *= 0x81dadef4bc2dd44d;
        hash ^= hash >> 33;
        hash %= self.hash_table_size as u64;

        // Now, see if the hash table already has an entry for the voxel. We'll
        // use quadratic probing when the hash table entry is already used for
        // another value; step stores the square root of the probe step.
        let mut step = 1;
        let mut n_probes = 0;
        loop {
            n_probes += 1;
            let entry = &self.hash_table[hash as usize];
            // Does the hash table entry at offset |hash| match the current point?
            let entry_packed_pos = entry.packed_pos.load(Ordering::Acquire);
            if entry_packed_pos == packed_pos {
                // Yes! Most of the time, there should already by a light
                // sampling distribution available.
                let mut dist = entry.distribution.load(Ordering::Acquire);
                if dist.is_null() {
                    // Rarely, another thread will have already done a lookup
                    // at this point, found that there isn't a sampling
                    // distribution, and will already be computing the
                    // distribution for the point.  In this case, we spin until
                    // the sampling distribution is ready.  We assume that this
                    // is a rare case, so don't do anything more sophisticated
                    // than spinning.
                    loop {
                        dist = entry.distribution.load(Ordering::Acquire);
                        if !dist.is_null() {
                            break;
                        }
                        // spin :-(. If we were fancy, we'd have any threads
                        // that hit this instead help out with computing the
                        // distribution for the voxel...
                    }
                }
                unsafe {
                    // We have a valid sampling distribution.
                    return dist.as_ref().unwrap();
                }
            } else if entry_packed_pos != INVALID_PACKED_POS {
                // The hash table entry we're checking has already been
                // allocated for another voxel. Advance to the next entry with
                // quadratic probing.
                hash += step * step;
                if hash >= self.hash_table_size as u64 {
                    hash %= self.hash_table_size as u64;
                }
                step += 1;
            } else {
                // We have found an invalid entry. (Though this may have
                // changed since the load into entryPackedPos above.)  Use an
                // atomic compare/exchange to try to claim this entry for the
                // current position.
                if entry
                       .packed_pos
                       .compare_exchange_weak(INVALID_PACKED_POS,
                                              packed_pos,
                                              Ordering::SeqCst,
                                              Ordering::SeqCst)
                       .is_ok() {
                    // Success; we've claimed this position for this voxel's
                    // distribution. Now compute the sampling distribution and
                    // add it to the hash table. As long as packedPos has been
                    // set but the entry's distribution pointer is nullptr, any
                    // other threads looking up the distribution for this voxel
                    // will spin wait until the distribution pointer is
                    // written.
                    let distrib = Box::new(self.compute_distribution(&pi));
                    let distrib_ptr = Box::into_raw(distrib);
                    entry.distribution.store(distrib_ptr, Ordering::Release);
                    unsafe {
                        return distrib_ptr.as_ref().unwrap();
                    }
                }
            }
        }
    }
}

// TODO implement Drop?

struct HashEntry {
    pub packed_pos: AtomicU64,
    pub distribution: AtomicPtr<Distribution1D>,
}
