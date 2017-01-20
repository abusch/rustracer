use minifb;

use ::Dim;
use film::Film;

pub trait DisplayUpdater {
    fn update(&mut self, film: &Film);
}

pub struct MinifbDisplayUpdater {
    window: minifb::Window,
}

impl MinifbDisplayUpdater {
    pub fn new(dim: Dim) -> MinifbDisplayUpdater {
        MinifbDisplayUpdater {
            window: minifb::Window::new("Rustracer",
                                        dim.0 as usize,
                                        dim.1 as usize,
                                        minifb::WindowOptions::default())
                .expect("Unable to open a window"),
        }
    }
}

impl DisplayUpdater for MinifbDisplayUpdater {
    fn update(&mut self, film: &Film) {
        let buffer: Vec<u32> = film.render()
            .iter()
            .map(|p| {
                let rgb = p.to_srgb();
                (rgb[0] as u32) << 16 | (rgb[1] as u32) << 8 | (rgb[2] as u32)

            })
            .collect();

        self.window.update_with_buffer(&buffer[..]);
    }
}

// minifb::Window is not Send because of some callback it holds, but we need MinifbDisplayUpdater
// to be so we can send it to the thread collecting the tiles. This is a bit naughty but since it
// is only moved to some other thread once at the beginning, this should be fine... (I hope!)
unsafe impl Send for MinifbDisplayUpdater {}

pub struct NoopDisplayUpdater;

impl DisplayUpdater for NoopDisplayUpdater {
    fn update(&mut self, _film: &Film) {}
}
