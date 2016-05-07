use std::io::{self, Read};
use std::fs;
use std::path::{Path, PathBuf};

use flate2::FlateReadExt;

use repo::ar;
use repo::deb;
use tar;

#[derive(Debug)]
pub struct PackageMeta {
    pub filename: PathBuf,
    pub arch: String,
    pub version: String,
}

fn error(text: &'static str) -> io::Error {
    return io::Error::new(io::ErrorKind::Other, text);
}

pub fn gather_metadata<P: AsRef<Path>>(p: P) -> io::Result<PackageMeta> {
    let path = p.as_ref();
    let mut arch = try!(ar::Archive::new(try!(fs::File::open(path))));
    {
        let mut buf = String::with_capacity(4);
        let mut member = try!(arch.read_file("debian-binary"));
        try!(member.read_to_string(&mut buf));
        if buf.trim() != "2.0" {
            return Err(error("Unsupported deb format"));
        }
    }
    let member = try!(arch.read_file("control.tar.gz"));
    let mut arch = tar::Archive::new(try!(member.gz_decode()));
    for entry in try!(arch.entries()) {
        let entry = try!(entry);
        if try!(entry.path()) == Path::new("control") {
            let hash = try!(deb::parse_control(entry));
            return Ok(PackageMeta {
                filename: path.to_path_buf(),
                arch: try!(hash.get(&"Architecture".into()).map(Clone::clone)
                    .ok_or(error("No architecture in deb package meta"))),
                version: try!(hash.get(&"Version".into()).map(Clone::clone)
                    .ok_or(error("No version in deb package meta"))),
            });
        }
    }
    return Err(error("No metadata found"));
}
