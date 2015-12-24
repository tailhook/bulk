use std::io;
use std::io::{Write, Seek, SeekFrom};

pub struct ArArchive<T:Write+Seek>(T);

struct ArMember<'a, T:Write+Seek+'a> {
    position: u64,
    current_size: usize,
    defined_size: usize,
    archive: &'a mut ArArchive<T>,
}

impl<'a, T:Write+Seek+'a> Write for ArMember<'a, T> {
    fn write(&mut self, data: &[u8]) -> Result<usize, io::Error> {
        match self.archive.0.write(data) {
            Ok(x) => {
                self.current_size += x;
                Ok(x)
            }
            Err(e) => Err(e),
        }
    }
    fn flush(&mut self) -> Result<(), io::Error> {
        self.archive.0.flush()
    }
}

impl<'a, T:Write+Seek+'a> Drop for ArMember<'a, T> {
    fn drop(&mut self) {
        if self.current_size != self.defined_size {
            // Since we had already written there, we assume that we can't
            // fail. Anyway crashing is probably okay in this case
            let cur_pos = self.archive.0.seek(SeekFrom::Current(0)).unwrap();
            self.archive.0.seek(SeekFrom::Start(self.position + 48)).unwrap();
            write!(self.archive.0,
                "{size:<10}", size=self.current_size).unwrap();
            self.archive.0.seek(SeekFrom::Start(cur_pos)).unwrap();
        }
        if self.current_size % 2 != 0 {
            self.archive.0.write(b"\n").unwrap();
        }
    }
}

impl<T:Write+Seek> ArArchive<T> {
    pub fn new(mut file: T) -> Result<ArArchive<T>, io::Error> {
        try!(file.write_all(b"!<arch>\n"));
        Ok(ArArchive(file))
    }
    pub fn add<'x>(&'x mut self, filename: &str,
        filemtime: u32, uid: u32, gid: u32,
        mode: u32, size: usize) -> Result<ArMember<'x, T>, io::Error>
    {
        assert!(filename.len() <= 16);
        assert!(uid <= 999999);
        assert!(gid <= 999999);
        assert!(mode <= 99999999);
        assert!(size <= 9999999999);
        let pos = try!(self.0.seek(SeekFrom::Current(0)));
        try!(write!(&mut self.0,
            "{name:<16}{mtime:<12}{uid:<6}{gid:<6}{mode:<8o}{size:<10}`\n",
            name=filename, mtime=filemtime, uid=uid, gid=gid, mode=mode,
            size=size));
        Ok(ArMember {
            position: pos,
            current_size: 0,
            defined_size: size,
            archive: self,
        })
    }
}
