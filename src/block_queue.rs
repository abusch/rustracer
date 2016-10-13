use std::io;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

use na::Point2;

#[derive(Debug)]
pub struct Block {
    start: Point2<usize>,
    current: Point2<usize>,
    end: Point2<usize>,
}

impl Block {
    fn new(start: (usize, usize), size: usize) -> Block {
        Block {
            start: Point2::new(start.0, start.1),
            current: Point2::new(start.0, start.1),
            end: Point2::new(start.0 + size - 1, start.1 + size - 1),
        }
    }
}

impl Iterator for Block {
    type Item = Point2<usize>;

    fn next(&mut self) -> Option<Point2<usize>> {
        if self.current.x > self.end.x || self.current.y > self.end.y {
            None
        } else {

            let cur = self.current;

            if self.current.x == self.end.x {
                self.current.x = self.start.x;
                self.current.y += 1;
            } else {
                self.current.x += 1;
            }

            Some(cur)
        }
    }
}

pub struct BlockQueue {
    dims: (usize, usize),
    block_size: usize,
    counter: AtomicUsize,
    pub num_blocks: usize,
}

impl BlockQueue {
    pub fn new(dims: (usize, usize), block_size: usize) -> BlockQueue {
        BlockQueue {
            dims: dims,
            block_size: block_size,
            counter: ATOMIC_USIZE_INIT,
            num_blocks: (dims.0 / block_size) * (dims.1 / block_size),
        }
    }

    pub fn next(&self) -> Option<Block> {
        let c = self.counter.fetch_add(1, Ordering::AcqRel);
        if c >= self.num_blocks {
            None
        } else {
            let num_blocks_width = self.dims.0 / self.block_size;
            Some(Block::new((c % num_blocks_width * self.block_size,
                             c / num_blocks_width * self.block_size),
                            self.block_size))
        }
    }

    pub fn report_progress(&self) {
        print!("\rRendering block {}/{}...  ",
               self.counter.load(Ordering::Relaxed),
               self.num_blocks);
        io::stdout().flush().expect("Could not flush stdout");;
    }
}
