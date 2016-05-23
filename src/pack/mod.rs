mod metadata;
mod ar;
mod tar;
mod deb;

use std::io;
use std::io::{stdout, stderr, Write};
use std::env;
use std::fs::{File, create_dir, rename, remove_file};
use std::path::{Path, PathBuf};
use std::process::exit;

use argparse::{ArgumentParser, Parse};
use tar::{Builder as Archive};
use flate2::{GzBuilder, Compression};
use scan_dir;

use config::{Config, Metadata};
use self::ar::{ArArchive, SIZE_AUTO};
use self::metadata::populate;
use self::tar::ArchiveExt;
use self::deb::format_deb_control;


fn write_deb(dest: &Path, dir: &Path, meta: &Metadata)
    -> Result<(), io::Error>
{
    let mtime = env::var("SOURCE_DATE_EPOCH").ok()
        .and_then(|x| x.parse().ok()).unwrap_or(1);
    let file = try!(File::create(&dest));
    let mut ar = try!(ArArchive::new(file));

    try!(ar.add("debian-binary", mtime, 0, 0, 0o100644, 4)
        .and_then(|mut f| f.write_all(b"2.0\n")));

    {
        let control = try!(ar.add("control.tar.gz",
            mtime, 0, 0, 0o100644, SIZE_AUTO));
        let creal = GzBuilder::new().write(control, Compression::Best);
        let mut arch = Archive::new(creal);
        try!(arch.append_blob("control", mtime,
            &format_deb_control(&meta)));
        try!(arch.finish());
    }
    {
        let data = try!(ar.add("data.tar.gz",
            mtime, 0, 0, 0o100644, SIZE_AUTO));
        let dreal = GzBuilder::new().write(data, Compression::Best);
        let mut files = try!(scan_dir::ScanDir::files().skip_backup(true)
            .walk(dir, |iter| {
                iter.map(|(entry, _name)| {
                    entry.path().strip_prefix(dir).unwrap().to_path_buf()})
                    .collect::<Vec<_>>()
            }).map_err(|errs| io::Error::new(io::ErrorKind::InvalidData,
                errs.iter().map(ToString::to_string).collect::<Vec<_>>()[..]
                    .join("\n"))));
        files.sort();
        let mut arch = Archive::new(dreal);
        for fpath in files {
            try!(arch.append_file_at(dir, fpath, mtime));
        }
        try!(arch.finish());
    }
    Ok(())
}

fn _pack(config: &Path, dir: &Path, destdir: &Path) -> Result<(), String> {
    let cfg = try!(Config::parse_file(config));
    let meta = try!(populate(&cfg));
    // TODO(tailhook) not only debian
    let dest = destdir.join(format!("{}-{}_{}.deb",
        meta.name, meta.version, meta.architecture));
    if !destdir.exists() {
        try!(create_dir(&destdir)
            .map_err(|e| format!("Can't create destination dir: {}", e)));
    }

    let tmpname = dest.with_extension(".deb.tmp");
    try!(write_deb(&tmpname, dir, &meta)
         .map_err(|e| format!("Error writing deb: {}", e)));
    if dest.exists() {
        try!(remove_file(&dest)
            .map_err(|e| format!("Can't remove old package: {}", e)));
    }
    try!(rename(&tmpname, &dest)
        .map_err(|e| format!("Can't rename deb to target place: {}", e)));
    Ok(())
}


pub fn pack(args: Vec<String>) {
    let mut config = PathBuf::from("package.yaml");
    let mut dir = PathBuf::from("pkg");
    let mut destdir = PathBuf::from("dist");
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut config)
            .add_option(&["-c", "--config"], Parse,
                "Package configuration file");
        ap.refer(&mut dir)
            .add_option(&["-d", "--dir"], Parse,
                "Directory that will be a root of filesystem in a package");
        ap.refer(&mut destdir)
            .add_option(&["-D", "--dest-dir"], Parse,
                "Directory to put package to");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => exit(x),
        }
    }

    match _pack(&config, &dir, &destdir) {
        Ok(()) => {}
        Err(text) => {
            writeln!(&mut stderr(), "Error: {}", text).ok();
            exit(1);
        }
    }
}
