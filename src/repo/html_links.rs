use std::path::{Path, PathBuf};

pub struct Repository {
    base_dir: PathBuf,
}

impl Repository {
    pub fn new(path: impl AsRef<Path>) -> Repository {
        Repository {
            base_dir: path.as_ref().to_path_buf(),
        }
    }
}
