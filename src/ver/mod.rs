use std::io::{self, stdout, stderr, Write, BufWriter, BufReader};
use std::fs::{File, remove_file, rename};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::process::{Command, exit};
use std::collections::HashMap;

use config::{Config};
use version::Version;
use argparse::{ArgumentParser, Parse, StoreTrue, List};

use self::scanner::{Scanner, Lines, Iter};

mod scanner;


fn _get(config: &Path, dir: &Path) -> Result<Version<String>, Box<Error>> {
    let cfg = try!(Config::parse_file(&config));
    get(&cfg, dir)
}

pub fn get(cfg: &Config, dir: &Path) -> Result<Version<String>, Box<Error>> {
    for item in &cfg.versions {
        if item.partial_version.is_some() { // can't get from partial version
            continue;
        }
        let scanner = try!(scanner::Scanner::new(&item)
            .map_err(|e| format!("One of the regexps is wrong: {} for {:#?}",
                e, cfg)));
        for filename in item.file.iter().chain(&item.files) {
            let file = match File::open(&dir.join(&filename)) {
                Ok(i) => BufReader::new(i),
                Err(ref e) if e.kind() == io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            };
            let mut iter = scanner.start();
            for res in Lines::iter(file) {
                let (lineno, line) = try!(res);
                match iter.line(lineno, &line) {
                    Some((start, end)) => {
                        return Ok(Version(line[start..end].to_string()));
                    }
                    None => {}
                }
            }
            try!(iter.error());
        }
    }
    return Err("Version not found".into());
}

fn _check(config: &Path, dir: &Path) -> Result<bool, Box<Error>> {
    let cfg = try!(Config::parse_file(&config));
    let mut prev: Option<String> = None;
    let mut result = true;
    // partial versions go after full
    let lst = cfg.versions.iter().filter(|x| x.partial_version.is_none())
        .chain(cfg.versions.iter().filter(|x| x.partial_version.is_some()));
    for item in lst {
        let scanner = try!(scanner::Scanner::new(&item)
            .map_err(|e| format!("One of the regexps is wrong: {} for {:#?}",
                e, cfg)));
        for filename in item.file.iter().chain(&item.files) {
            let file = match File::open(&dir.join(&filename)) {
                Ok(i) => BufReader::new(i),
                Err(ref e) if e.kind() == io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            };
            let mut iter = scanner.start();
            for res in Lines::iter(file) {
                let (lineno, line) = try!(res);
                match iter.line(lineno, &line) {
                    Some((start, end)) => {
                        let ver = &line[start..end];
                        println!("{}:{}: (v{}) {}",
                            filename.display(), lineno, ver,
                            line.trim_right());
                        if let Some(ref pver) = prev {
                            let cver = scanner.partial.as_ref()
                                .map(|re| re.captures(pver)
                                    .expect("partial-version must match")
                                    .at(0).unwrap())
                                .unwrap_or(&pver[..]);
                            if cver != ver {
                                result = false;
                                writeln!(&mut stderr(),
                                    "{}:{}: version conflict {} != {}",
                                    filename.display(), lineno,
                                    ver, cver).ok();
                            }
                        } else {
                            prev = Some(ver.to_string());
                        }
                    }
                    None => {}
                }
            }
            try!(iter.error());
        }
    }
    if prev.is_none() {
        Err(format!("No version found").into())
    } else {
        Ok(result)
    }
}

fn _set(config: &Path, dir: &Path, version: &str, dry_run: bool, force: bool)
    -> Result<String, Box<Error>>
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
    -> Result<String, Box<Error>>
{
    let mut prev: Option<String> = None;
    let mut result = Ok(());
    let mut scanners = HashMap::new();
    for item in &cfg.versions {
        let scanner = try!(scanner::Scanner::new(&item)
            .map_err(|e| format!("One of the regexps is wrong: {} for {:#?}",
                e, cfg)));
        for file in item.file.iter().chain(&item.files) {
            scanners.entry(file.clone())
            .or_insert_with(Vec::new)
            .push(scanner.clone())
        }
    }
    for (filename, list) in scanners {
        let filename = dir.join(filename);
        let mut tmp = filename.as_os_str().to_owned();
        tmp.push(".tmp");
        let tmp = tmp.into();
        let file = match File::open(&filename) {
            Ok(i) => BufReader::new(i),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        };

        let mut out = BufWriter::new(try!(File::create(&tmp)));
        files.push((tmp, filename.to_path_buf()));

        let mut scanners = list.iter().map(Scanner::start).collect::<Vec<_>>();
        try!(Lines::iter(file).map(|res| {
            let (lineno, line) = try!(res);
            let nline = scanners.iter_mut().fold(line, |line, citer| {
                match citer.line(lineno, &line) {
                    Some((start, end)) => {
                        let ver = &line[start..end];

                        let partver = citer.scanner().partial.as_ref()
                            .map(|re| re.captures(version)
                                .expect("partial-version must match")
                                .at(0).unwrap())
                            .unwrap_or(&version[..]);

                        let nline = String::from(&line[..start])
                            + partver + &line[end..];

                        println!("{}:{}: (v{} -> v{}) {}",
                            filename.display(), lineno, ver, partver,
                            nline.trim_right());
                        if let Some(ref pver) = prev {
                            let cver = citer.scanner().partial.as_ref()
                                .map(|re| re.captures(pver)
                                    .expect("partial-version must match")
                                    .at(0).unwrap())
                                .unwrap_or(&pver[..]);
                            if cver != ver {
                                let msg = format!(
                                    "{}:{}: version conflict {} != {}",
                                    filename.display(), lineno,
                                    ver, cver);
                                if force {
                                    println!("{}", msg);
                                } else {
                                    result = Err(msg.into());
                                }
                            }
                        } else {
                            prev = Some(ver.to_string());
                        }
                        nline
                    }
                    None => line,
                }
            });
            out.write_all(nline.as_bytes())
        }).collect::<Result<Vec<()>, _>>());
        try!(scanners.into_iter().map(Iter::error)
            .collect::<Result<Vec<_>, _>>());
    }
    if let Some(ver) = prev {
        if let Err(e) = result {
            Err(e)
        } else {
            Ok(ver)
        }
    } else {
        Err(format!("No version found").into())
    }
}

pub fn get_version(args: Vec<String>) {
    let mut config = PathBuf::from("bulk.yaml");
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
    let mut config = PathBuf::from("bulk.yaml");
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
        Ok(_) => {}
        Err(text) => {
            writeln!(&mut stderr(), "Error: {}", text).ok();
            exit(1);
        }
    }
}

pub fn check_version(args: Vec<String>) {
    let mut config = PathBuf::from("bulk.yaml");
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

pub fn with_version(args: Vec<String>) {
    let mut config = PathBuf::from("bulk.yaml");
    let mut dir = PathBuf::from(".");
    let mut version = String::new();
    let mut cmdline = Vec::<String>::new();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut config)
            .add_option(&["-c", "--config"], Parse,
                "Package configuration file");
        ap.refer(&mut dir)
            .add_option(&["--base-dir"], Parse, "
                Base directory for all paths in config. \
                Current working directory by default.");
        ap.refer(&mut version)
            .add_argument("version", Parse, "Target version")
            .required();
        ap.refer(&mut cmdline)
            .add_argument("cmd", List, "Command and arguments")
            .required();
        ap.stop_on_first_argument(true);

        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => exit(x),
        }
    }

    let old = match _set(&config, &dir, &version, false, false) {
        Ok(ver) => ver,
        Err(text) => {
            writeln!(&mut stderr(), "Error: {}", text).ok();
            exit(99);
        }
    };

    let mut cmd = Command::new(cmdline.remove(0));
    cmd.args(&cmdline);
    let result = cmd.status();

    match _set(&config, &dir, &old, false, false) {
        Ok(_) => {}
        Err(text) => {
            writeln!(&mut stderr(), "Error: {}", text).ok();
            exit(99);
        }
    }

    match result {
        Ok(s) => {
            if let Some(x) = s.code() {
                exit(x);
            } else {
                exit(98);
            }
        }
        Err(e) => {
            writeln!(&mut stderr(), "Error running command: {}", e).ok();
            exit(98);
        }
    }
}
