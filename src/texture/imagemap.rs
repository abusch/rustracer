use std::sync::Arc;
use std::path::Path;

use Point2i;
use interaction::SurfaceInteraction;
use imageio::read_image;
use mipmap::{MIPMap, WrapMode};
use spectrum::Spectrum;
use texture::{Texture, TextureMapping2D, UVMapping2D};

#[derive(Debug)]
pub struct ImageTexture {
    mapping: Box<TextureMapping2D + Send + Sync>,
    mipmap: Arc<MIPMap<Spectrum>>,
}

impl ImageTexture {
    pub fn new(path: &Path) -> ImageTexture {
        info!("Loading texture {}", path.display());
        let (res, pixels) = match read_image(path) {
            Ok((mut pixels, res)) => {
                // Flip image in y; texture coordinate space has (0,0) at the lower
                // left corner.
                for y in 0..res.y / 2 {
                    for x in 0..res.x {
                        let o1 = (y * res.x + x) as usize;
                        let o2 = ((res.y - 1 - y) * res.x + x) as usize;
                        pixels.swap(o1, o2);
                    }
                }

                (res, pixels)
            }
            Err(e) => {
                warn!(
                    "Could not open texture file. Using grey texture instead: {}",
                    e
                );
                (Point2i::new(1, 1), vec![Spectrum::grey(0.18)])
            }
        };

        ImageTexture {
            mapping: Box::new(UVMapping2D::new(1.0, 1.0, 0.0, 0.0)),
            mipmap: Arc::new(MIPMap::new(&res, &pixels[..], false, 0.0, WrapMode::Repeat)),
        }
    }
}

impl Texture<Spectrum> for ImageTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> Spectrum {
        let st = self.mapping.map(si);
        // TODO Call correct lookup method once we have ray differentials
        self.mipmap.lookup(&st, 0.0)
    }
}

#[test]
fn load_texture() {
    ImageTexture::new(&Path::new("lines.png"));
    assert!(true);
}
