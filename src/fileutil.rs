use std::sync::Mutex;
use std::path::{Path, PathBuf};

lazy_static!{
    static ref SEARCH_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
}

pub fn set_search_directory(d: &str) {
    let mut dir = SEARCH_DIR.lock().unwrap();
    dir.get_or_insert(PathBuf::from(d));
    info!("Set search directory to {}", d);
}

pub fn directory_containing(file: &str) -> &str {
    let path = Path::new(file);
    let parent = path.parent().expect(
        format!(
            "Failed to get the parent directory of the input file {}",
            file
        ).as_ref(),
    );


    parent
        .to_str()
        .expect("Filename contained invalid UTF-8 characters")
}

pub fn resolve_filename(filename: &str) -> String {
    let search_directory = SEARCH_DIR.lock().unwrap();
    if search_directory.is_none() || filename == "" {
        return filename.to_owned();
    } else {
        let mut buf = (*search_directory).clone().unwrap(); //PathBuf::from(*search_directory);
        buf.push(filename);
        return buf.as_path()
            .canonicalize()
            .unwrap()
            .to_str()
            .expect("Filename contained invalid UTF-8 characters")
            .to_owned();
    }
}
