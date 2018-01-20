use std::io::{self, Read};
use std::fs::File;
use std::path::Path;

use sha2::{Sha256, Digest};


pub fn hash_stream<D: Digest, R: Read>(hash: &mut D, reader: &mut R)
    -> Result<(), io::Error>
{
    let mut buf = [0u8; 8*1024];
    loop {
        let len = match reader.read(&mut buf[..]) {
            Ok(0) => break,
            Ok(len) => len,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        hash.input(&buf[..len]);
    }
    Ok(())
}

pub fn hash_file<F: AsRef<Path>>(filename: F) -> io::Result<String> {
    let mut sha256 = Sha256::new();
    let mut file = try!(File::open(filename.as_ref()));
    try!(hash_stream(&mut sha256, &mut file));
    return Ok(format!("{:x}", sha256.result()));
}
