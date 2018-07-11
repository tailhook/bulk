use std::io;
use std::fs::{File, symlink_metadata, read_link};
use std::path::Path;
use std::os::unix::fs::PermissionsExt;

use tar;


pub trait ArchiveExt {
    fn append_blob<P: AsRef<Path>>(&mut self, name: P, mtime: u32, data: &[u8])
        -> Result<(), io::Error>;
    fn append_file_at<P: AsRef<Path>, Q: AsRef<Path>>(&mut self,
        dir: P, path: Q, mtime: u32)
        -> Result<(), io::Error>;
}

impl<T: io::Write> ArchiveExt for tar::Builder<T> {
    fn append_blob<P: AsRef<Path>>(&mut self, name: P, mtime: u32, data: &[u8])
        -> Result<(), io::Error>
    {
        let mut head = tar::Header::new_gnu();
        head.set_mtime(mtime as u64);
        head.set_size(data.len() as u64);
        head.set_mode(0o644);
        head.set_cksum();
        self.append_data(&mut head, name, &mut io::Cursor::new(&data))
    }
    /// This does same as Builder::append_file, but has no mtime/size/owner
    /// information which we explicitly have chosen to omit
    ///
    /// Silently skips things that are neither files nor symlinks
    fn append_file_at<P: AsRef<Path>, Q: AsRef<Path>>(&mut self,
        dir: P, path: Q, mtime: u32)
        -> Result<(), io::Error>
    {
        let path = path.as_ref();
        let fullpath = dir.as_ref().join(path);
        let meta = try!(symlink_metadata(&fullpath));

        let mut head = tar::Header::new_gnu();
        head.set_mtime(mtime as u64);

        if meta.file_type().is_file() {
            head.set_entry_type(tar::EntryType::Regular);
            let mut file = try!(File::open(&fullpath));
            head.set_size(meta.len() as u64);
            head.set_mode(meta.permissions().mode());
            head.set_cksum();
            self.append_data(&mut head, &path, &mut file)
        } else if meta.file_type().is_symlink() {
            head.set_entry_type(tar::EntryType::Symlink);
            let lnk = try!(read_link(&fullpath));
            head.set_size(0);
            head.set_mode(meta.permissions().mode());
            try!(head.set_link_name(lnk));
            head.set_cksum();
            self.append_data(&mut head, &path, &mut io::empty())
        } else if meta.file_type().is_dir() {
            head.set_entry_type(tar::EntryType::Directory);
            head.set_size(0);
            head.set_mode(meta.permissions().mode());
            head.set_cksum();
            self.append_data(&mut head, &path, &mut io::empty())
        } else {
            // Silently skip as documented
            Ok(())
        }
    }
}
