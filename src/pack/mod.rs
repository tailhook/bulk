mod ar;
mod tar;
mod deb;

use std::io;
use std::io::{stdout, stderr, Write};
use std::env;
use std::fs::{File, create_dir, rename, remove_file};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::process::exit;

use argparse::{ArgumentParser, Parse, ParseOption};
use tar::{Builder as Archive};
use flate2::{GzBuilder, Compression};
use scan_dir;

use ver;
use config::{Config, Metadata};
use self::ar::{ArArchive, SIZE_AUTO};
use self::tar::ArchiveExt;
use self::deb::format_deb_control;


fn write_deb(dest: &Path, dir: &Path, meta: &Metadata, version: &String)
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
        let mut buf = Vec::with_capacity(1024);
        try!(format_deb_control(&mut buf, &meta, version, "amd64"));
        try!(arch.append_blob("control", mtime, &buf));
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

fn _pack(config: &Path, dir: &Path, destdir: &Path, version: Option<String>)
    -> Result<(), Box<Error>>
{
    let cfg = try!(Config::parse_file(config));

    let version = if let Some(ver) = version {
        ver
    } else {
        try!(ver::get(&cfg, Path::new("."))).0
    };


    let ref meta = cfg.metadata;
    // TODO(tailhook) not only debian
    let dest = destdir.join(format!("{}-{}_{}.deb",
        meta.name, version, "amd64"));
    if !destdir.exists() {
        try!(create_dir(&destdir)
            .map_err(|e| format!("Can't create destination dir: {}", e)));
    }

    let tmpname = dest.with_extension(".deb.tmp");
    try!(write_deb(&tmpname, dir, &meta, &version)
         .map_err(|e| format!("Error writing deb: {}", e)));
    if dest.exists() {
        try!(remove_file(&dest)
            .map_err(|e| format!("Can't remove old package: {}", e)));
    }
    try!(rename(&tmpname, &dest)
        .map_err(|e| format!("Can't rename deb to target place: {}", e)));
    println!("Written {}", dest.display());
    Ok(())
}


pub fn pack(args: Vec<String>) {
    let mut config = PathBuf::from("bulk.yaml");
    let mut dir = PathBuf::from("pkg");
    let mut destdir = PathBuf::from("dist");
    let mut version = None;
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
        ap.refer(&mut version)
            .add_option(&["--package-version"], ParseOption,
                "Force package version instead of discovering it.");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => exit(x),
        }
    }

    match _pack(&config, &dir, &destdir, version) {
        Ok(()) => {}
        Err(text) => {
            writeln!(&mut stderr(), "Error: {}", text).ok();
            exit(1);
        }
    }
}
