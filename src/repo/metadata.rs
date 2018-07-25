use std::io::{self, Read, BufReader};
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use failure::Error;
use libflate::gzip;
use regex::Regex;
use unicase::UniCase;

use config::{Config, RepositoryType};
use repo::ar;
use repo::deb;
use tar;

#[derive(Debug, PartialEq)]
pub struct PackageMeta {
    pub kind: RepositoryType,
    pub filename: PathBuf,
    pub name: String,
    pub arch: String,
    pub version: String,
    pub info: HashMap<UniCase<String>, String>,
}

lazy_static!{
    static ref DEFAULT_FILE_REGEX: Regex = Regex::new(
        r"^(?P<name>.*?)-(?P<version>v?\d[^.]*(?:\.\d[^.]*)*)\."
    ).expect("valid default file regex");
}

fn error(text: &'static str) -> io::Error {
    return io::Error::new(io::ErrorKind::Other, text);
}

pub fn gather_metadata(p: impl AsRef<Path>, cfg: &Config)
    -> Result<Option<PackageMeta>, Error>
{
    let p = p.as_ref();
    let fname = p.file_name().and_then(|x| x.to_str());
    match fname {
        Some(fname) if fname.ends_with(".deb") => {
            read_deb(p)
                .map_err(|e| format_err!("Error reading deb {:?}: {}", p, e))
                .map(Some)
        }
        Some(fname) if fname.ends_with(".tar.gz") => {
            let mut metadata = None;
            for repo in &cfg.repositories {
                if repo.kind == RepositoryType::HtmlLinks {
                    let regex = repo.match_filename.as_ref()
                        .unwrap_or_else(|| &*DEFAULT_FILE_REGEX);
                    if let Some(capt) = regex.captures(fname) {
                        let pkg = PackageMeta {
                            kind: RepositoryType::HtmlLinks,
                            filename: p.to_path_buf(),
                            name: capt.name("name")
                                .ok_or_else(|| format_err!(
                                    "no `name` group in match_filename"))?
                                .as_str().to_string(),
                            arch: "".into(),  // should be None?
                            version: capt.name("version")
                                .ok_or_else(|| format_err!(
                                    "no `version` group in match_filename"))?
                                .as_str().to_string(),
                            info: HashMap::new(),
                        };
                        if let Some(old) = metadata {
                            if old != pkg {
                                warn!("Conflicting package metadata: \
                                    {:?} vs {:?}", old, pkg);
                            }
                        }
                        metadata = Some(pkg);
                    }
                }
            }
            Ok(metadata)
        }
        Some(_) | None => {
            return Err(format_err!("Unknown package type {:?}", p));
        }
    }
}
pub fn read_deb(path: &Path) -> io::Result<PackageMeta> {
    let buf = BufReader::new(fs::File::open(path)?);
    let mut arch = ar::Archive::new(buf)?;
    {
        let mut buf = String::with_capacity(4);
        let mut member = try!(arch.read_file("debian-binary"));
        try!(member.read_to_string(&mut buf));
        if buf.trim() != "2.0" {
            return Err(error("Unsupported deb format"));
        }
    }
    let member = try!(arch.read_file("control.tar.gz"));
    let mut arch = tar::Archive::new(try!(gzip::Decoder::new(member)));
    for entry in try!(arch.entries()) {
        let entry = try!(entry);
        if try!(entry.path()) == Path::new("control") {
            let control = try!(deb::parse_control(entry));
            if control.len() != 1 {
                return Err(error("Wrong control file in package"));
            }
            let hash = control.into_iter().next().unwrap();
            return Ok(PackageMeta {
                kind: RepositoryType::Debian,
                filename: path.to_path_buf(),
                name: try!(hash.get(&"Package".into()).map(Clone::clone)
                    .ok_or(error("No package name in deb package meta"))),
                arch: try!(hash.get(&"Architecture".into()).map(Clone::clone)
                    .ok_or(error("No architecture in deb package meta"))),
                version: try!(hash.get(&"Version".into()).map(Clone::clone)
                    .ok_or(error("No version in deb package meta"))),
                info: hash,
            });
        }
    }
    return Err(error("No metadata found"));
}
