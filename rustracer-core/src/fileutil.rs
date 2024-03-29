use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use log::debug;
use parking_lot::Mutex;

lazy_static! {
    static ref SEARCH_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
}

pub fn set_search_directory<P: AsRef<Path>>(d: P) {
    let d = d.as_ref();
    let mut dir = SEARCH_DIR.lock();
    dir.get_or_insert(PathBuf::from(d));
    debug!("Set search directory to {}", d.display());
}

pub fn directory_containing<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();

    path.canonicalize()
        .unwrap()
        .parent()
        .unwrap_or_else(|| {
            panic!(
                "Failed to get the parent directory of the input file {}",
                path.display()
            )
        })
        .to_owned()
}

pub fn resolve_filename(filename: &str) -> String {
    debug!("Resolving filename {}", filename);
    let search_directory = SEARCH_DIR.lock();
    if search_directory.is_none() || filename.is_empty() || Path::new(filename).is_absolute() {
        filename.to_owned()
    } else {
        let mut buf = (*search_directory).clone().unwrap(); //PathBuf::from(*search_directory);
        buf.push(filename);
        buf.as_path()
            .canonicalize()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_owned()))
            .unwrap_or_else(|| filename.to_owned())
    }
}

pub fn has_extension<P: AsRef<Path>>(filename: P, extension: &str) -> bool {
    filename
        .as_ref()
        .extension()
        .map(|e| e == extension)
        .unwrap_or(false)
}
