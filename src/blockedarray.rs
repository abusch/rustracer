use std::ops::{Index, IndexMut};
use std::fmt::Debug;

use num::{zero, Zero};

/// The logarithm (in base-2) of the block size. This ensures that the block size is a power of 2.
/// In this case, 3 means a block size of 8.
const LOG_BLOCK_SIZE: usize = 3;
const BLOCK_SIZE: usize = 1 << LOG_BLOCK_SIZE;

#[derive(Debug)]
pub struct BlockedArray<T> {
    u_res: usize,
    v_res: usize,
    u_blocks: usize,
    log_block_size: usize,
    block_size: usize,
    data: Vec<T>,
}

impl<T> BlockedArray<T>
where
    T: Copy + Zero + Debug,
{
    pub fn new(u_res: usize, v_res: usize) -> BlockedArray<T> {
        let data = vec![zero(); round_up(u_res) * round_up(v_res)];

        BlockedArray {
            u_res: u_res,
            v_res: v_res,
            u_blocks: round_up(u_res) >> LOG_BLOCK_SIZE,
            log_block_size: LOG_BLOCK_SIZE,
            block_size: BLOCK_SIZE,
            data: data,
        }
    }

    pub fn new_from(u_res: usize, v_res: usize, d: &[T]) -> BlockedArray<T> {
        let mut ba = Self::new(u_res, v_res);

        for u in 0..u_res {
            for v in 0..v_res {
                ba[(u, v)] = d[v * u_res + u]
            }
        }

        ba
    }

    pub fn u_size(&self) -> usize {
        self.u_res
    }

    pub fn v_size(&self) -> usize {
        self.v_res
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    pub fn block(&self, a: usize) -> usize {
        a >> self.log_block_size
    }

    pub fn offset(&self, a: usize) -> usize {
        a & (self.block_size() - 1)
    }
}

fn round_up(x: usize) -> usize {
    (x + BLOCK_SIZE - 1) & !(BLOCK_SIZE - 1)
}

impl<T> Index<(usize, usize)> for BlockedArray<T>
where
    T: Copy,
    T: Zero,
    T: Debug,
{
    type Output = T;

    fn index(&self, i: (usize, usize)) -> &T {
        let (u, v) = i;
        let bu = self.block(u);
        let bv = self.block(v);
        let ou = self.offset(u);
        let ov = self.offset(v);
        let offset = self.block_size() * self.block_size() * (self.u_blocks * bv + bu)
            + self.block_size() * ov + ou;
        &self.data[offset]
    }
}

impl<T> IndexMut<(usize, usize)> for BlockedArray<T>
where
    T: Copy,
    T: Zero,
    T: Debug,
{
    fn index_mut(&mut self, i: (usize, usize)) -> &mut T {
        let (u, v) = i;
        let bu = self.block(u);
        let bv = self.block(v);
        let ou = self.offset(u);
        let ov = self.offset(v);
        let offset = self.block_size() * self.block_size() * (self.u_blocks * bv + bu)
            + self.block_size() * ov + ou;
        &mut self.data[offset]
    }
}
