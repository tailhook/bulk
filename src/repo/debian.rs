use std::io::{self, Write, BufWriter};
use std::fs::{File, create_dir_all, rename, copy, metadata};
use std::num::ParseIntError;
use std::path::{PathBuf, Path};
use std::collections::{BTreeSet, BTreeMap, HashMap};

use time::now_utc;
use sha2::{Sha256, Digest};
use unicase::UniCase;
use quick_error::ResultExt;

use config;
use version::Version;
use hash_file::hash_file;
use deb_ext::WriteDebExt;
use repo::metadata::PackageMeta;
use repo::deb::parse_control;
use repo::ConflictResolution;


#[derive(Debug)]
pub struct Release {
    codename: String,
    architectures: BTreeSet<String>,
    components: BTreeSet<String>,
    sha256: BTreeMap<String, (u64, String)>,
}

#[derive(Debug)]
pub struct Package {
    name: String,
    version: Version<String>,
    architecture: String,
    filename: PathBuf,
    size: u64,
    sha256: String,
    metadata: BTreeMap<UniCase<String>, String>,
}

#[derive(Debug)]
pub struct Index {
    packages: BTreeMap<(String, String), BTreeMap<Version<String>, Package>>,
    limit: Option<usize>,
}

#[derive(Debug)]
struct FileInfo {
    path: PathBuf,
    sha256: String,
    size: u64,
}

#[derive(Debug)]
pub struct Component<'a>(&'a mut Index,
                         &'a mut HashMap<PathBuf, FileInfo>);

#[derive(Debug)]
pub struct Repository {
    root: PathBuf,
    suites: HashMap<String, Release>,
    components: HashMap<(String, String, String), Index>,
    files: HashMap<PathBuf, FileInfo>,
}

quick_error! {
    #[derive(Debug)]
    pub enum ReleaseFileRead {
        Io(err: io::Error) {
            from()
            description("io error")
            display("io error: {}", err)
        }
        AbsentField(field: &'static str) {
            description("required field is absent")
            display("field {:?} is absent", field)
        }
        ExcessiveControlData {
            description("more than one control data in Releases file")
        }
        FileSize(err: ParseIntError) {
            from()
            display("error parsing file size: {}", err)
            description("error parsing file size")
        }
        InvalidHashLine {
            description("one of the lines of SHA256 has invalid format")
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum PackagesRead {
        Io(err: io::Error) {
            from()
            description("io error")
            display("io error: {}", err)
        }
        AbsentField(field: &'static str) {
            description("required field is absent")
            display("field {:?} is absent", field)
        }
        FileSize(err: ParseIntError) {
            from()
            display("error parsing file size: {}", err)
            description("error parsing file size")
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum RepositoryError {
        Config(val: &'static str) {
            display("debian repository misconfiguration: {}", val)
        }
        PackageConflict(pkg: Package) {
            description("package we are trying to add is already in repo")
            display("package {}-{}-{} is already in repository",
                pkg.name, pkg.version, pkg.architecture)
        }
        Release(path: PathBuf, err: ReleaseFileRead) {
            description("can't open Release file")
            display("can't open {:?}: {}", path, err)
            context(path: AsRef<Path>, err: ReleaseFileRead)
                -> (path.as_ref().to_path_buf(), err)
        }
        Packages(path: PathBuf, err: PackagesRead) {
            description("can't open Packages file")
            display("can't open {:?}: {}", path, err)
            context(path: AsRef<Path>, err: PackagesRead)
                -> (path.as_ref().to_path_buf(), err)
        }
    }
}

impl Release {
    fn read(path: &Path) -> Result<Release, ReleaseFileRead> {
        use self::ReleaseFileRead::*;
        let mut datas = try!(parse_control(try!(File::open(path))));
        if datas.len() != 1 {
            return Err(ExcessiveControlData);
        }
        let mut data = datas.pop().unwrap();
        let codename = try!(data.remove(&"Codename".into())
                            .ok_or(AbsentField("Codename")));
        let architectures = try!(data.remove(&"Architectures".into())
                               .ok_or(AbsentField("Architectures")))
                               .split_whitespace()
                               .map(ToString::to_string).collect();
        let components = try!(data.remove(&"Components".into())
                               .ok_or(AbsentField("Components")))
                               .split_whitespace()
                               .map(ToString::to_string).collect();
        let files = data.get(&"SHA256".into()).map(|x| &x[..]).unwrap_or("")
            .split("\n");
        let mut hashsums = BTreeMap::new();
        for line in files {
            let line = line.trim();
            if line == "" { continue; }
            let mut iter = line.split_whitespace();
            match (iter.next(), iter.next(), iter.next(), iter.next()) {
                (Some(hash), Some(size), Some(fname), None) => {
                    let size = try!(size.parse());
                    hashsums.insert(fname.to_string(),
                                    (size, hash.to_string()));
                }
                _ => {
                    return Err(InvalidHashLine);
                }
            }
        }
        Ok(Release {
            codename: codename,
            architectures: architectures,
            components: components,
            sha256: hashsums,
        })
    }
    fn output<W: Write>(&self, out: &mut W) -> io::Result<()> {
        try!(out.write_kv("Codename", &self.codename));
        // TODO(tailhook) better use latest date from packages
        // to make rebuilding the indices reproducible
        try!(out.write_kv("Date", &format!("{}", now_utc().rfc822z())));
        try!(out.write_kv("Architectures",
            &self.architectures.iter().map(|x| &x[..])
                .collect::<Vec<&str>>()[..].join(" ")));
        try!(out.write_kv("Components",
            &self.components.iter().map(|x| &x[..])
                .collect::<Vec<&str>>()[..].join(" ")));
        try!(out.write_kv_lines("SHA256",
            self.sha256.iter().map(|(fname, &(size, ref hash))| {
                format!("{} {} {}", hash, size, fname)
            })));
        Ok(())
    }
}

impl Index {
    fn read(path: &Path) -> Result<Index, PackagesRead> {
        use self::PackagesRead::*;
        let mut coll = BTreeMap::new();
        let items = try!(parse_control(try!(File::open(path))));
        for mut control in items.into_iter() {
            let name = try!(control.remove(&"Package".into())
                           .ok_or(AbsentField("Package")));
            let version = Version(try!(control.remove(&"Version".into())
                           .ok_or(AbsentField("Version"))));
            let architecture = try!(control.remove(&"Architecture".into())
                           .ok_or(AbsentField("Architecture")));
            coll.entry((name.clone(), architecture.clone()))
                .or_insert_with(BTreeMap::new)
                .insert(version.clone(), Package {
                    name: name,
                    version: version,
                    architecture: architecture,
                    filename: try!(control.remove(&"Filename".into())
                               .ok_or(AbsentField("Filename"))).into(),
                    size: try!(try!(control.remove(&"Size".into())
                               .ok_or(AbsentField("Size"))).parse()),
                    sha256: try!(control.remove(&"SHA256".into())
                               .ok_or(AbsentField("SHA256"))),
                    metadata: control.into_iter().collect(),
                });
        }
        Ok(Index {
            packages: coll,
            limit: None,
        })
    }
    fn output<W: Write>(&self, out: &mut W) -> io::Result<()> {
        for versions in self.packages.values() {
            for p in versions.values() {
                try!(out.write_kv("Package", &p.name));
                try!(out.write_kv("Version", p.version.as_ref()));
                try!(out.write_kv("Architecture", &p.architecture));
                try!(out.write_kv("Filename",
                    &p.filename.to_str().expect("package name should be ascii")));
                try!(out.write_kv("SHA256", &p.sha256));
                try!(out.write_kv("Size", &format!("{}", p.size)));
                for (k, v) in &p.metadata {
                    if *k != UniCase::new("Package") &&
                       *k != UniCase::new("Version") &&
                       *k != UniCase::new("Architecture")
                    {
                        try!(out.write_kv(k, v));
                    }
                }
                try!(out.write_all(b"\n"));
            }
        }
        Ok(())
    }
    pub fn new() -> Index {
        Index {
          packages: BTreeMap::new(),
          limit: None,
        }
    }
}


impl<'a> Component<'a> {
    pub fn add_package(&mut self, pack: &PackageMeta,
        on_conflict: ConflictResolution)
        -> Result<(), RepositoryError>
    {
        let info = self.1.entry(pack.filename.clone())
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
            metadata: pack.info.iter()
                .map(|(k, v)| (k.clone(), v.clone())).collect(),
        };
        let component_arch = (pack.name.clone(), pack.arch.clone());
        let versions = self.0.packages.entry(component_arch)
            .or_insert_with(BTreeMap::new);
        if versions.contains_key(&pkg.version) {
            use self::ConflictResolution::*;
            match on_conflict {
                Error => Err(RepositoryError::PackageConflict(pkg)),
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

impl Repository {
    pub fn new(base_dir: &Path) -> Repository {
        Repository {
            root: base_dir.to_path_buf(),
            suites: HashMap::new(),
            components: HashMap::new(),
            files: HashMap::new(),
        }
    }
    pub fn open(&mut self, repo: &config::Repository, arch: &str)
        -> Result<Component, RepositoryError>
    {
        let suite = repo.suite.as_ref()
            .ok_or_else(|| RepositoryError::Config("suite must be specified"))?;
        let component = repo.component.as_ref().ok_or_else(|| {
            RepositoryError::Config("component must be specified")
        })?;
        if !self.suites.contains_key(suite) {
            let release_file = self.root.join("dists").join(suite)
                               .join("Release");
            let rel = if release_file.exists() {
                try!(Release::read(&release_file).context(&release_file))
            } else {
                Release {
                    codename: suite.clone(),
                    architectures: BTreeSet::new(),
                    components: BTreeSet::new(),
                    sha256: BTreeMap::new(),
                }
            };
            self.suites.insert(suite.clone(), rel);
        }
        let s = self.suites.get_mut(suite).unwrap();
        s.architectures.insert(String::from(arch));
        s.components.insert(component.clone());

        let triple = (suite.clone(), component.clone(), String::from(arch));
        if !self.components.contains_key(&triple) {
            let packages_file = self.root.join("dists").join(suite)
                .join(component).join(format!("binary-{}/Packages", arch));
            let mut packages = if packages_file.exists() {
                Index::read(&packages_file).context(&packages_file)?
            } else {
                Index::new()
            };
            packages.limit = repo.keep_releases;
            self.components.insert(triple.clone(), packages);
        }
        let packages = self.components.get_mut(&triple).unwrap();
        Ok(Component(packages, &mut self.files))
    }
    fn trim(&mut self) {
        for (_, ref mut pkgs)
            in self.components.iter_mut()
        {
            if let Some(limit) = pkgs.limit {
              for (_, ref mut collection) in pkgs.packages.iter_mut() {
                  while collection.len() > limit {
                      let smallest = collection.keys()
                          .next().unwrap().clone();
                      collection.remove(&smallest);
                  }
              }
            }

        }
    }
    pub fn write(mut self) -> io::Result<()> {
        if self.suites.len() == 0 && self.components.len() == 0 {
            return Ok(());
        }
        self.trim();

        let mut tempfiles = Vec::new();
        for ((suite, cmp, arch), pkg) in self.components {
            let dir = self.root
                .join("dists").join(&suite).join(&cmp)
                .join(format!("binary-{}", arch));
            try!(create_dir_all(&dir));
            let tmp = dir.join("Packages.tmp");
            let mut buf = Vec::with_capacity(16384);
            try!(pkg.output(&mut buf));
            try!(File::create(&tmp).and_then(|mut f| f.write_all(&buf)));
            tempfiles.push((tmp, dir.join("Packages")));

            let mut hash = Sha256::new();
            hash.input(&buf);

            self.suites.get_mut(&suite).expect("suite already created")
            .sha256.insert(format!("{}/binary-{}/Packages", cmp, arch),
                (buf.len() as u64, format!("{:x}", hash.result())));
        }
        for (_, suite) in self.suites {
            let dir = self.root.join("dists").join(&suite.codename);
            let tmp = dir.join("Release.tmp");
            let ref mut buf = BufWriter::new(try!(File::create(&tmp)));
            try!(suite.output(buf));
            tempfiles.push((tmp, dir.join("Release")));
        }
        for (ref src, ref info) in self.files {
            let realdest = self.root.join(&info.path);
            let tmpname = realdest.with_file_name(
                String::from(realdest.file_name().unwrap().to_str().unwrap())
                + ".tmp");
            try!(create_dir_all(&realdest.parent().unwrap()));
            try!(copy(src, &tmpname));
            tempfiles.push((tmpname, realdest));
        }
        for &(ref a, ref b) in &tempfiles {
            try!(rename(a, b));
        }
        Ok(())
    }
}
