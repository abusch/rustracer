use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use std::fmt;

use na::Point2;

use Point2i;
use bounds::Bounds2i;

/// A block of pixels that a thread is responsible for rendering (i.e a bucket).
#[derive(Debug)]
pub struct Block {
    pub start: Point2i,
    current: Point2i,
    pub end: Point2i,
}

impl Block {
    pub fn new(start: (u32, u32), size: u32) -> Block {
        Block {
            start: Point2i::new(start.0 as i32, start.1 as i32),
            current: Point2i::new(start.0 as i32, start.1 as i32),
            end: Point2i::new(start.0 as i32 + size as i32, start.1 as i32 + size as i32),
        }
    }

    /// Return the area of this block in pixels (i.e. number of pixels this
    /// block covers)
    pub fn area(&self) -> u32 {
        (self.end.x - self.start.x) as u32 * (self.end.y - self.start.y) as u32
    }

    pub fn bounds(&self) -> Bounds2i {
        Bounds2i::from_points(&self.start, &self.end)
    }
}

impl Iterator for Block {
    type Item = Point2i;

    fn next(&mut self) -> Option<Point2i> {
        if self.current.x >= self.end.x || self.current.y >= self.end.y {
            None
        } else {

            let cur = self.current;

            if self.current.x == self.end.x - 1 {
                self.current.x = self.start.x;
                self.current.y += 1;
            } else {
                self.current.x += 1;
            }

            Some(cur)
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} → {}", self.start, self.end)
    }
}

pub struct BlockQueue {
    pub dims: (u32, u32),
    pub block_size: u32,
    counter: AtomicUsize,
    pub num_blocks: u32,
}

impl BlockQueue {
    pub fn new(dims: (u32, u32), block_size: u32) -> BlockQueue {
        let xblocks = (dims.0 as f32 / block_size as f32).ceil() as u32;
        let yblocks = (dims.1 as f32 / block_size as f32).ceil() as u32;
        BlockQueue {
            dims: dims,
            block_size: block_size,
            counter: ATOMIC_USIZE_INIT,
            num_blocks: xblocks * yblocks,
        }
    }

    pub fn next(&self) -> Option<Block> {
        let c = self.counter.fetch_add(1, Ordering::AcqRel) as u32;
        if c >= self.num_blocks {
            None
        } else {
            let num_blocks_width = self.dims.0 / self.block_size;
            Some(Block::new((c % num_blocks_width * self.block_size,
                             c / num_blocks_width * self.block_size),
                            self.block_size))
        }
    }
}

impl Iterator for BlockQueue {
    type Item = Block;

    fn next(&mut self) -> Option<Block> {
        BlockQueue::next(self)
    }
}

#[test]
fn test_area() {
    let block = Block::new((12, 12), 8);
    assert_eq!(block.area(), 64);
}

#[test]
fn test_iter() {
    let block = Block::new((12, 12), 8);
    let pixels: Vec<Point2i> = block.into_iter().collect();

    assert_eq!(pixels.len(), 64);
    assert_eq!(pixels[0].x, 12);
    assert_eq!(pixels[0].y, 12);
    assert_eq!(pixels[63].x, 19);
    assert_eq!(pixels[63].y, 19);
}

#[test]
fn test_queue_iter() {
    let queue = BlockQueue::new((100, 100), 8);
    let blocks: Vec<Block> = queue.into_iter().collect();

    // 100 is not a multiple of 8, so make sure we generate enough blocks to cover the whole image.
    // In this case, we need 13 * 13.
    assert_eq!(blocks.len(), 169);
}

#[test]
fn test_power_of_two() {
    let queue = BlockQueue::new((96, 96), 8);
    let blocks: Vec<Block> = queue.into_iter().collect();

    assert_eq!(blocks.len(), 144);
}
