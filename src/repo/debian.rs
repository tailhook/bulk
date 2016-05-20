use std::io::{self, Write, BufWriter};
use std::fs::{File, create_dir_all};
use std::path::{PathBuf, Path};
use std::collections::{BTreeSet, BTreeMap, HashMap};

use sha2::sha2::Sha256;
use sha2::digest::Digest;
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
    components: HashMap<(String, String, String), Packages>,
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
    pub fn open(&mut self, suite: &str, component: &str, arch: &str)
        -> io::Result<&mut Packages>
    {
        let s = self.suites.entry(String::from(suite))
            .or_insert_with(|| Release {
                codename: String::from(suite),
                architectures: BTreeSet::new(),
                components: BTreeSet::new(),
                sha256: BTreeMap::new(),
            });
        s.architectures.insert(String::from(arch));
        s.components.insert(String::from(component));
        Ok(self.components.entry(
            (String::from(suite), String::from(component), String::from(arch))
        ).or_insert_with(Packages::new))
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
            let tmp = dir.join("InRelease.tmp");
            let ref mut buf = BufWriter::new(try!(File::create(&tmp)));
            try!(suite.output(buf));
            tempfiles.push((tmp, dir.join("InRelease")));
        }
        Ok(())
    }
}
