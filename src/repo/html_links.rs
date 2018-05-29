use std::path::{Path, PathBuf};

use failure::Error;

use repo::metadata::PackageMeta;
use repo::ConflictResolution;

pub struct Repository {
    base_dir: PathBuf,
}

pub struct Subrepository<'a> {
    repo: &'a mut Repository,
}

impl Repository {
    pub fn new(path: impl AsRef<Path>) -> Repository {
        Repository {
            base_dir: path.as_ref().to_path_buf(),
        }
    }
    pub fn open(&mut self, index: &Path, files: &Path)
        -> Result<Subrepository, Error>
    {
        Ok(Subrepository {
            repo: self,
        })
    }
}

impl<'a> Subrepository<'a> {
    pub fn add_package(&mut self, pack: &PackageMeta,
        on_conflict: ConflictResolution)
        -> Result<(), Error>
    {
        unimplemented!();
    }
}
