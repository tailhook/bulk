use std::io::{self, Read, BufReader};
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use failure::Error;
use libflate::gzip;
use unicase::UniCase;

use repo::ar;
use repo::deb;
use tar;

#[derive(Debug)]
pub struct PackageMeta {
    pub filename: PathBuf,
    pub name: String,
    pub arch: String,
    pub version: String,
    pub info: HashMap<UniCase<String>, String>,
}

fn error(text: &'static str) -> io::Error {
    return io::Error::new(io::ErrorKind::Other, text);
}

pub fn gather_metadata(p: impl AsRef<Path>) -> Result<PackageMeta, Error> {
    let p = p.as_ref();
    let fname = p.file_name().and_then(|x| x.to_str());
    match fname {
        Some(fname) if fname.ends_with(".deb") => {
            read_deb(p)
                .map_err(|e| format_err!("Error reading deb {:?}: {}", p, e))
        }
        Some(fname) if fname.ends_with(".tar.gz") => {
            unimplemented!("tar.gz");
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
