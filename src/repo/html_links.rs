use std::path::{Path, PathBuf};
use std::fs::{File, create_dir_all, rename, copy, metadata};
use std::collections::{BTreeMap, HashMap};

use failure::Error;

use version::Version;
use hash_file::hash_file;
use repo::metadata::PackageMeta;
use repo::ConflictResolution;

pub struct Repository {
    base_dir: PathBuf,
    repos: HashMap<PathBuf, BTreeMap<Version<String>, Package>>,
    file_info: HashMap<PathBuf, FileInfo>,
}

pub struct Subrepository<'a> {
    repo: &'a mut Repository,
    index: &'a Path,
    files: &'a Path,
}

#[derive(Debug)]
struct Package {
    name: String,
    version: Version<String>,
    architecture: String,
    filename: PathBuf,
    size: u64,
    sha256: String,
}

#[derive(Debug)]
struct FileInfo {
    path: PathBuf,
    sha256: String,
    size: u64,
}

impl Repository {
    pub fn new(path: impl AsRef<Path>) -> Repository {
        Repository {
            base_dir: path.as_ref().to_path_buf(),
            file_info: HashMap::new(),
            repos: HashMap::new(),
        }
    }
    pub fn open<'a>(&'a mut self, index: &'a Path, files: &'a Path)
        -> Result<Subrepository<'a>, Error>
    {
        Ok(Subrepository {
            repo: self,
            index: index,
            files: files,
        })
    }
    pub fn write(mut self) -> Result<(), Error> {
        if self.repos.is_empty() {
            return Ok(())
        }
        unimplemented!();
    }
}

impl<'a> Subrepository<'a> {
    pub fn add_package(&mut self, pack: &PackageMeta,
        on_conflict: ConflictResolution)
        -> Result<(), Error>
    {
        let info = self.repo.file_info.entry(pack.filename.clone())
            .or_insert_with(|| {
                let filename = pack.filename.file_name()
                               .expect("package path should have a filename");
                let tpath = Path::new("pool")
                    .join(pack.name.chars().take(1).collect::<String>())
                    .join(&pack.name)
                    .join(filename);

                // TODO(tailhook) report errors in some nicer way
                let hash = hash_file(&pack.filename)
                    .expect("read file");
                let size = metadata(&pack.filename)
                    .expect("read file")
                    .len();

                FileInfo {
                    path: tpath,
                    sha256: hash,
                    size: size,
                }
            });
        let pkg = Package {
            name: pack.name.clone(),
            version: Version(pack.version.clone()),
            architecture: pack.arch.clone(),
            filename: info.path.clone(),
            sha256: info.sha256.clone(),
            size: info.size,
        };
        let versions = self.repo.repos.entry(self.index.to_path_buf())
            .or_insert_with(BTreeMap::new);
        if versions.contains_key(&pkg.version) {
            use self::ConflictResolution::*;
            match on_conflict {
                Error => bail!("package {}-{}-{} is already in repository",
                                pkg.name, pkg.version, pkg.architecture),
                Keep => Ok(()),
                Replace => {
                    versions.insert(pkg.version.clone(), pkg);
                    Ok(())
                }
            }
        } else {
            versions.insert(pkg.version.clone(), pkg);
            Ok(())
        }
    }
}
