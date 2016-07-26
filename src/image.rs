use Dim;
use colour::Colourf;

pub struct Image {
    pub width: u32,
    pub height: u32,
    pixels: Vec<Colourf>
}

impl Image {
    pub fn new(dim: Dim) -> Image {
        let (w, h) = dim;
        let size = w as usize * h as usize;
        let mut buffer = Vec::with_capacity(size);
        buffer.resize(size, Colourf::black());

        Image {
            width: w,
            height: h,
            pixels: buffer
        }
    }

    pub fn write(&mut self, x: u32, y: u32, colour: Colourf) {
        self.pixels[(y * self.width + x) as usize] = colour;
    }

    pub fn buffer(&self) -> &[Colourf] {
        &self.pixels
    }
}


