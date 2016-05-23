use std::io::{self, Write, BufWriter};
use std::fs::{File, create_dir_all, rename, copy, metadata};
use std::num::ParseIntError;
use std::path::{PathBuf, Path};
use std::error::Error;
use std::collections::{BTreeSet, BTreeMap, HashMap};

use time::now_utc;
use sha2::sha2::Sha256;
use sha2::digest::Digest;
use unicase::UniCase;
use quick_error::ResultExt;

use hash_file::hash_file;
use deb_ext::WriteDebExt;
use repo::metadata::PackageMeta;
use repo::deb::parse_control;


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
    version: String,
    architecture: String,
    filename: PathBuf,
    size: u64,
    sha256: String,
    metadata: BTreeMap<UniCase<String>, String>,
}

#[derive(Debug)]
pub struct Packages(Vec<Package>);

#[derive(Debug)]
struct FileInfo {
    path: PathBuf,
    sha256: String,
    size: u64,
}

#[derive(Debug)]
pub struct Component<'a>(&'a mut Packages,
                         &'a mut HashMap<PathBuf, FileInfo>);

#[derive(Debug)]
pub struct Repository {
    root: PathBuf,
    suites: HashMap<String, Release>,
    components: HashMap<(String, String, String), Packages>,
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
    pub enum RepositoryError {
        Release(path: PathBuf, err: ReleaseFileRead) {
            description("can't open Release file")
            display("can't open {:?}: {}", path, err)
            context(path: AsRef<Path>, err: ReleaseFileRead)
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

impl Packages {
    fn output<W: Write>(&self, out: &mut W) -> io::Result<()> {
        for p in &self.0 {
            try!(out.write_kv("Package", &p.name));
            try!(out.write_kv("Version", &p.version));
            try!(out.write_kv("Architecture", &p.architecture));
            try!(out.write_kv("Filename",
                &p.filename.to_str().expect("package name should be ascii")));
            try!(out.write_kv("SHA256", &p.sha256));
            try!(out.write_kv("Size", &format!("{}", p.size)));
            for (k, v) in &p.metadata {
                if *k != "Package" && *k != "Version" && *k != "Architecture" {
                    try!(out.write_kv(k, v));
                }
            }
            try!(out.write_all(b"\n"));
        }
        Ok(())
    }
    pub fn new() -> Packages {
        Packages(Vec::new())
    }
}


impl<'a> Component<'a> {
    pub fn add_package(&mut self, pack: &PackageMeta)
        -> Result<(), Box<Error>>
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
        (self.0).0.push(Package {
            name: pack.name.clone(),
            version: pack.version.clone(),
            architecture: pack.arch.clone(),
            filename: info.path.clone(),
            sha256: info.sha256.clone(),
            size: info.size,
            metadata: pack.info.iter()
                .map(|(k, v)| (k.clone(), v.clone())).collect(),
        });
        Ok(())
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
    pub fn open(&mut self, suite: &str, component: &str, arch: &str)
        -> Result<Component, RepositoryError>
    {
        if !self.suites.contains_key(suite) {
            let release_file = self.root.join("dists").join(suite)
                               .join("Release");
            let rel = if release_file.exists() {
                try!(Release::read(&release_file).context(&release_file))
            } else {
                Release {
                    codename: String::from(suite),
                    architectures: BTreeSet::new(),
                    components: BTreeSet::new(),
                    sha256: BTreeMap::new(),
                }
            };
            self.suites.insert(String::from(suite), rel);
        }
        let s = self.suites.get_mut(suite).unwrap();
        s.architectures.insert(String::from(arch));
        s.components.insert(String::from(component));

        let packages = self.components.entry(
                (String::from(suite), String::from(component),
                 String::from(arch))
            ).or_insert_with(Packages::new);
        Ok(Component(packages, &mut self.files))
    }
    pub fn write(mut self) -> io::Result<()> {
        if self.suites.len() == 0 && self.components.len() == 0 {
            return Ok(());
        }

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
                (buf.len() as u64, hash.result_str()));
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
