use std::io::{stdout, stderr, Write, BufWriter};
use std::fs::{File, remove_file, rename};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::process::exit;

use config::{Config};
use version::Version;
use argparse::{ArgumentParser, Parse, StoreTrue};

mod scanner;


fn _get(config: &Path, dir: &Path) -> Result<Version<String>, Box<Error>> {
    let cfg = try!(Config::parse_file(&config));
    let mut result = Err("Version not found".into());
    for item in &cfg.versions {
        let scanner = try!(scanner::Scanner::new(&item)
            .map_err(|e| format!("One of the regexps is wrong: {} for {:#?}",
                e, cfg)));
        for file in item.file.iter().chain(&item.files) {
            let file = dir.join(file);
            try!(scanner.scan_file(&file, |lineno, line, capt| {
                if let Some(capt) = capt {
                    match capt.at(1) {
                        Some("") => {
                            result = Err(format!(
                                "{}:{}: captured empty version in: {}",
                                file.display(), lineno, line.trim_right())
                                .into());
                        }
                        Some(x) => {
                            result = Ok(Version(x.to_string()));
                        }
                        None => {
                            result = Err(format!(
                                "{}:{}: no version capture in regex {:?}",
                                file.display(), lineno, item.regex)
                                .into());
                        }
                    }
                    return false;
                }
                return true;
            }).map_err(|e| format!("{}: IO error: {}", file.display(), e)));
        }
    }
    return result;
}

fn _check(config: &Path, dir: &Path) -> Result<bool, Box<Error>> {
    let cfg = try!(Config::parse_file(&config));
    let mut prev = None;
    let mut result = true;
    for item in &cfg.versions {
        let scanner = try!(scanner::Scanner::new(&item)
            .map_err(|e| format!("One of the regexps is wrong: {} for {:#?}",
                e, cfg)));
        for file in item.file.iter().chain(&item.files) {
            let file = dir.join(file);
            try!(scanner.scan_file(&file, |lineno, line, capt| {
                if let Some(capt) = capt {
                    match capt.at(1) {
                        Some("") => {
                            result = false;
                            writeln!(&mut stderr(),
                                "{}:{}: captured empty version in: {}",
                                file.display(), lineno, line.trim_right())
                                .ok();
                        }
                        Some(ver) => {
                            println!("{}:{}: (v{}) {}",
                                file.display(), lineno, ver,
                                line.trim_right());
                            if let Some(ref pver) = prev {
                                if pver != ver {
                                    result = false;
                                    writeln!(&mut stderr(),
                                        "{}:{}: version conflict {} != {}",
                                        file.display(), lineno,
                                        ver, pver).ok();
                                }
                            } else {
                                prev = Some(ver.to_string());
                            }
                        }
                        None => {
                            result = false;
                            writeln!(&mut stderr(),
                                "{}:{}: no version capture in regex {:?}",
                                file.display(), lineno, item.regex).ok();
                        }
                    }
                }
                return true;
            }).map_err(|e| format!("{}: IO error: {}", file.display(), e)));
        }
    }
    if prev.is_none() {
        Err(format!("No version found").into())
    } else {
        Ok(result)
    }
}

fn _set(config: &Path, dir: &Path, version: &str, dry_run: bool, force: bool)
    -> Result<(), Box<Error>>
{
    let cfg = try!(Config::parse_file(&config));
    let mut buf = Vec::new();
    let mut result = _write_tmp(&cfg, dir, version, &mut buf, force);
    let mut iter = buf.into_iter();
    if !dry_run && result.is_ok() {
        for (tmp, dest) in iter.by_ref() {
            match rename(&tmp, &dest) {
                Ok(()) => {}
                Err(e) => {
                    result = Err(format!(
                        "Error renaming file {:?}: {}", tmp, e).into());
                    remove_file(&tmp)
                    .or_else(|e| writeln!(&mut stderr(),
                        "Error removing file {:?}: {}", tmp, e)).ok();
                }
            }
        }
    }
    for (tmp, _) in iter {
        remove_file(&tmp)
        .or_else(|e| writeln!(&mut stderr(),
            "Error removing file {:?}: {}", tmp, e)).ok();
    }
    return result;
}

fn _write_tmp(cfg: &Config, dir: &Path, version: &str,
    files: &mut Vec<(PathBuf, PathBuf)>, force: bool)
    -> Result<(), Box<Error>>
{
    let mut prev = None;
    let mut result = Ok(());
    for item in &cfg.versions {
        let scanner = try!(scanner::Scanner::new(&item)
            .map_err(|e| format!("One of the regexps is wrong: {} for {:#?}",
                e, cfg)));
        for file in item.file.iter().chain(&item.files) {
            let file = dir.join(file);
            let mut tmp = file.as_os_str().to_owned();
            tmp.push(".tmp");
            let tmp = tmp.into();
            let mut out = BufWriter::new(try!(File::create(&tmp)));
            files.push((tmp, file.to_path_buf()));
            try!(scanner.scan_file(&file, |lineno, line, capt| {
                if let Some(capt) = capt {
                    match capt.pos(1) {
                        Some((a, b)) if a == b => {
                            result = Err(format!(
                                "{}:{}: captured empty version in: {}",
                                file.display(), lineno, line.trim_right())
                                .into());
                            return false;
                        }
                        Some((start, end)) => {
                            let ver = &line[start..end];
                            let nline = String::from(&line[..start])
                                + version + &line[end..];
                            match out.write_all(nline.as_bytes()) {
                                Ok(_) => {}
                                Err(e) => {
                                    result = Err(e.into());
                                    return false;
                                }
                            }

                            println!("{}:{}: (v{} -> v{}) {}",
                                file.display(), lineno, ver, version,
                                nline.trim_right());
                            if let Some(ref pver) = prev {
                                if pver != ver {
                                    result = Err(format!(
                                        "{}:{}: version conflict {} != {}",
                                        file.display(), lineno,
                                        ver, pver).into());
                                    if !force {
                                        return false;
                                    }
                                }
                            } else {
                                prev = Some(ver.to_string());
                            }
                        }
                        None => {
                            result = Err(format!(
                                "{}:{}: no version capture in regex {:?}",
                                file.display(), lineno, item.regex).into());
                            return false;
                        }
                    }
                    return true;
                } else {
                    match out.write_all(line.as_bytes()) {
                        Ok(_) => {}
                        Err(e) => {
                            result = Err(e.into());
                            return false;
                        }
                    }
                }
                return true;
            }).map_err(|e| format!("{}: IO error: {}", file.display(), e)));
        }
    }
    if prev.is_none() {
        Err(format!("No version found").into())
    } else {
        result
    }
}

pub fn get_version(args: Vec<String>) {
    let mut config = PathBuf::from("package.yaml");
    let mut dir = PathBuf::from(".");
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut config)
            .add_option(&["-c", "--config"], Parse,
                "Package configuration file");
        ap.refer(&mut dir)
            .add_option(&["--base-dir"], Parse, "
                Base directory for all paths in config. \
                Current working directory by default.");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => exit(x),
        }
    }

    match _get(&config, &dir) {
        Ok(ver) => {
            println!("{}", ver);
        }
        Err(text) => {
            writeln!(&mut stderr(), "Error: {}", text).ok();
            exit(1);
        }
    }
}

pub fn set_version(args: Vec<String>) {
    let mut config = PathBuf::from("package.yaml");
    let mut dir = PathBuf::from(".");
    let mut version = String::new();
    let mut dry_run = false;
    let mut force = false;
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut config)
            .add_option(&["-c", "--config"], Parse,
                "Package configuration file");
        ap.refer(&mut dir)
            .add_option(&["--base-dir"], Parse, "
                Base directory for all paths in config. \
                Current working directory by default.");
        ap.refer(&mut dry_run)
            .add_option(&["--dry-run"], StoreTrue, "
                Don't write version, just show changes");
        ap.refer(&mut force)
            .add_option(&["--force"], StoreTrue, "
                Write version even if previous values are inconsistent");
        ap.refer(&mut version)
            .add_argument("version", Parse, "Target version")
            .required();

        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => exit(x),
        }
    }

    match _set(&config, &dir, &version, dry_run, force) {
        Ok(()) => {}
        Err(text) => {
            writeln!(&mut stderr(), "Error: {}", text).ok();
            exit(1);
        }
    }
}

pub fn check_version(args: Vec<String>) {
    let mut config = PathBuf::from("package.yaml");
    let mut dir = PathBuf::from(".");
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut config)
            .add_option(&["-c", "--config"], Parse,
                "Package configuration file");
        ap.refer(&mut dir)
            .add_option(&["--base-dir"], Parse, "
                Base directory for all paths in config. \
                Current working directory by default.");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => exit(x),
        }
    }

    match _check(&config, &dir) {
        Ok(val) => {
            exit(if val { 0 } else { 1 });
        }
        Err(text) => {
            writeln!(&mut stderr(), "Error: {}", text).ok();
            exit(1);
        }
    }
}
