use std::io;
use std::fs::{File, metadata, read_link};
use std::path::Path;
use std::os::unix::fs::PermissionsExt;

use tar;


pub trait ArchiveExt {
    fn append_blob<P: AsRef<Path>>(&self, name: P, mtime: u32, data: &[u8])
        -> Result<(), io::Error>;
    fn append_file_at<P: AsRef<Path>, Q: AsRef<Path>>(&self,
        dir: P, path: Q, mtime: u32)
        -> Result<(), io::Error>;
}

impl<T: io::Write> ArchiveExt for tar::Archive<T> {
    fn append_blob<P: AsRef<Path>>(&self, name: P, mtime: u32, data: &[u8])
        -> Result<(), io::Error>
    {
        let mut head = tar::Header::new();
        try!(head.set_path(name));
        head.set_mtime(mtime as u64);
        head.set_size(data.len() as u64);
        head.set_mode(0o644);
        head.set_cksum();
        self.append(&head, &mut io::Cursor::new(&data))
    }
    /// This does same as Archive::append_file, but has no mtime/size/owner
    /// information which we explicitly have chosen to omit
    ///
    /// Silently skips things that are neither files nor symlinks
    fn append_file_at<P: AsRef<Path>, Q: AsRef<Path>>(&self,
        dir: P, path: Q, mtime: u32)
        -> Result<(), io::Error>
    {
        let path = path.as_ref();
        let fullpath = dir.as_ref().join(path);
        let meta = try!(metadata(&fullpath));

        let mut head = tar::Header::new();
        try!(head.set_path(path));
        head.set_mtime(mtime as u64);

        if meta.file_type().is_file() {
            let mut file = try!(File::open(&fullpath));
            head.set_size(meta.len() as u64);
            head.set_mode(meta.permissions().mode());
            head.set_cksum();
            self.append(&head, &mut file)
        } else if meta.file_type().is_symlink() {
            let lnk = try!(read_link(&fullpath));
            head.set_size(0);
            head.set_mode(meta.permissions().mode());
            try!(head.set_link_name(lnk));
            head.set_cksum();
            self.append(&head, &mut io::Cursor::new(b""))
        } else {
            // Silently skip as documented
            Ok(())
        }
    }
}
