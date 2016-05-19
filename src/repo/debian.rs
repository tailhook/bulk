use std::io::{self, Write};
use std::path::{PathBuf, Path};
use std::collections::{BTreeSet, BTreeMap, HashMap};

use unicase::UniCase;

use deb_ext::WriteDebExt;
use repo::metadata::PackageMeta;


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
    metadata: BTreeMap<UniCase<String>, String>,
}

#[derive(Debug)]
pub struct Packages(Vec<Package>);

#[derive(Debug)]
pub struct Repository {
    root: PathBuf,
    suites: HashMap<String, Release>,
    components: HashMap<(String, String), Packages>,
}

impl Release {
    fn output<W: Write>(&self, out: &mut W) -> io::Result<()> {
        try!(out.write_kv("Codename", &self.codename));
        try!(out.write_kv("Architectures",
            &self.architectures.iter().map(|x| &x[..])
                .collect::<Vec<&str>>()[..].join(" ")));
        try!(out.write_kv("Components",
            &self.components.iter().map(|x| &x[..])
                .collect::<Vec<&str>>()[..].join(" ")));
        try!(out.write_kv_lines("SHA256",
            self.sha256.iter().map(|(fname, &(size, ref hash))| {
                format!("\n{} {} {}", hash, size, fname)
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
            for (k, v) in &p.metadata {
                if *k != "Package" && *k != "Version" && *k != "Architecture" {
                    try!(out.write_kv(k, v));
                }
            }
            try!(out.write_all(b"\n"));
        }
        Ok(())
    }
    pub fn add_package(&mut self, pack: &PackageMeta) {
        self.0.push(Package {
            name: pack.name.clone(),
            version: pack.version.clone(),
            architecture: pack.arch.clone(),
            metadata: pack.info.iter()
                .map(|(k, v)| (k.clone(), v.clone())).collect(),
        })
    }
    pub fn new() -> Packages {
        Packages(Vec::new())
    }
}

impl Repository {
    pub fn new(base_dir: &Path) -> Repository {
        Repository {
            root: base_dir.to_path_buf(),
            suites: HashMap::new(),
            components: HashMap::new(),
        }
    }
    pub fn open(&mut self, suite: &str, component: &str)
        -> io::Result<&mut Packages>
    {
        Ok(self.components.entry(
            (String::from(suite), String::from(component))
        ).or_insert_with(Packages::new))
    }
}
