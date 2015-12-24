mod metadata;
mod ar;

use std::io;
use std::io::{stdout, stderr, Write};
use std::env;
use std::fs::{File, create_dir, rename, remove_file};
use std::path::{Path, PathBuf};
use std::process::exit;

use argparse::{ArgumentParser, Parse};
use tar::{Archive, Header};
use flate2::{GzBuilder, Compression};

use config::Config;
use self::ar::ArArchive;
use self::metadata::{populate, Metadata};

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
        let control = try!(ar.add("control.tar.gz", mtime, 0, 0, 0o100644,
            9999999999));
        let creal = GzBuilder::new().write(control, Compression::Best);
        let arch = Archive::new(creal);
        let data = format!(concat!(
            "Package: {name}\n",
            "Version: {version}\n",
            "Architecture: {arch}\n",
            "Maintainer: tin\n",  // TODO(tailhook)
            "Description: {short_description}\n",  // TODO(tailhook)
            ), name=meta.name, version=meta.version, arch=meta.architecture,
               short_description=meta.short_description);
        let bytes = data.as_bytes();
        let mut head = Header::new();
        try!(head.set_path("control"));
        head.set_mtime(mtime as u64);
        head.set_size(bytes.len() as u64);
        head.set_mode(0o644);
        head.set_cksum();
        try!(arch.append(&head, &mut io::Cursor::new(bytes)));
        try!(arch.finish());
    }
    {
        let data = try!(ar.add("data.tar.gz", mtime, 0, 0, 0o100644,
            9999999999));
        let dreal = GzBuilder::new().write(data, Compression::Best);
        let arch = Archive::new(dreal);
        let data = "hello";
        let bytes = data.as_bytes();
        let mut head = Header::new();
        try!(head.set_path("usr/share/text.txt"));
        head.set_mtime(mtime as u64);
        head.set_size(bytes.len() as u64);
        head.set_mode(0o644);
        head.set_cksum();
        try!(arch.append(&head, &mut io::Cursor::new(bytes)));
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

    println!("Mdata {:?} -> {:?}: {:#?}", dir, dest, meta);

    // Reproducible builds spec

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
        ap.stop_on_first_argument(true);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => exit(x),
        }
    }

    match _pack(&config, &dir, &destdir) {
        Ok(()) => {}
        Err(text) => {
            writeln!(&mut stderr(), "{}", text).ok();
            exit(1);
        }
    }
}
