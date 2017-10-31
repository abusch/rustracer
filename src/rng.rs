use ONE_MINUS_EPSILON;

const PCG32_DEFAULT_STATE: u64 = 0x853c49e6748fea9b;
const PCG32_DEFAULT_STREAM: u64 = 0xda3e39cb94b95bdb;
const PCG32_MULT: u64 = 0x5851f42d4c957f2d;

#[derive(Copy, Clone)]
pub struct RNG {
    state: u64,
    inc: u64,
}

impl RNG {
    pub fn new() -> RNG {
        RNG {
            state: PCG32_DEFAULT_STATE,
            inc: PCG32_DEFAULT_STREAM,
        }
    }

    pub fn uniform_u32(&mut self) -> u32 {
        let oldstate = self.state;
        self.state = oldstate * PCG32_MULT + self.inc;
        let xorshifted = (((oldstate >> 18) ^ oldstate) >> 27) as u32;
        let rot = (oldstate >> 59) as u32;

        (xorshifted >> rot) | (xorshifted << ((!rot + 1) & 31))
    }

    pub fn uniform_u32_bounded(&mut self, b: u32) -> u32 {
        let threshold = (!b + 1) & b;
        loop {
            let r = self.uniform_u32();
            if r >= threshold {
                return r % b;
            }
        }
    }

    pub fn uniform_f32(&mut self) -> f32 {
        (self.uniform_u32() as f32 * 2.3283064365386963e-10).min(ONE_MINUS_EPSILON)
    }

    pub fn set_sequence(&mut self, seed: u64) {
        self.state = 0;
        self.inc = (seed << 1) | 1;
        let _ = self.uniform_u32();
        self.state += PCG32_DEFAULT_STATE;
        let _ = self.uniform_u32();
    }
}

impl Default for RNG {
    fn default() -> RNG {
        RNG::new()
    }
}
