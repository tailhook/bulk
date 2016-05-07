use std::io::{self, Read, Take};
use std::str;
use std::path::Path;
use std::os::unix::ffi::OsStrExt;

pub struct Archive<T:Read>(T, bool);


fn error(text: &'static str) -> io::Error {
    return io::Error::new(io::ErrorKind::Other, text);
}


impl<T:Read> Archive<T> {
    pub fn new(mut stream: T) -> io::Result<Archive<T>> {
        let mut sig = [0u8; 8];
        try!(stream.read(&mut sig));
        if &sig != b"!<arch>\n" {
            return Err(error("Archive signature is wrong"));
        }
        Ok(Archive(stream, false))
    }
    /// Reads file with known name
    ///
    /// Since we only read debian archives, it's good enough
    pub fn read_file<'x, P: AsRef<Path>>(&mut self, name: P)
        -> io::Result<Take<&mut T>>
    {
        return self._read_file(name.as_ref());
    }
    fn _read_file<'x>(&mut self, name: &Path) -> io::Result<Take<&mut T>> {
        let mut buf = [0u8; 61];
        let head = {
            if self.1 {
                let bytes = try!(self.0.read(&mut buf[..61]));
                if bytes != 61 {
                    return Err(error("Premature end of file"));
                }
                &buf[1..60]
            } else {
                let bytes = try!(self.0.read(&mut buf[..60]));
                if bytes != 60 {
                    return Err(error("Premature end of file"));
                }
                &buf[0..60]
            }
        };
        if &head[58..60] != b"`\n" {
            return Err(error("Invalid file format"));
        }
        let fnameend = head[..16].iter().position(|&x| x == b' ')
                        .unwrap_or(16);
        if &head[..fnameend] != name.as_os_str().as_bytes() {
            return Err(error("Unexpected archive member"));
        }
        let size = try!(
            str::from_utf8(&head[48..58]).ok()
            .and_then(|x| x.trim().parse().ok())
            .ok_or_else(|| error("Invalid file size")));
        self.1 = size % 1 == 1;
        Ok((&mut self.0).take(size))
    }
}
