use std::io;
use std::path::{Path, PathBuf};


#[derive(Debug)]
pub struct PackageMeta {
    pub filename: PathBuf,
}


pub fn gather_metadata<P: AsRef<Path>>(p: P) -> io::Result<PackageMeta> {
    Ok(PackageMeta {
        filename: p.as_ref().to_path_buf(),
    })
}
