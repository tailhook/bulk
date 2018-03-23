use std::io::{self, Read, BufReader};
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

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

pub fn gather_metadata<P: AsRef<Path>>(p: P) -> io::Result<PackageMeta> {
    let path = p.as_ref();
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
