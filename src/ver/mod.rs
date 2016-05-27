use std::io::{stdout, stderr, Write};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::process::exit;

use regex::Regex;
use config::{Config};
use version::Version;
use argparse::{ArgumentParser, Parse};

mod parse;


fn _get(config: &Path, dir: &Path) -> Result<Version<String>, Box<Error>> {
    let cfg = try!(Config::parse_file(&config));
    for item in &cfg.versions {
        for file in item.file.iter().chain(&item.files) {
            if let Some(x) = try!(parse::get_first(file, item)) {
                return Ok(x);
            }
        }
    }
    return Err("Version not found".into());
}

fn _set(config: &Path, dir: &Path, version: &str) -> Result<(), Box<Error>> {
    let cfg = try!(Config::parse_file(&config));
    return Err("not implemented".into());
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
            .add_argument("version", Parse, "Target version");

        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => exit(x),
        }
    }

    match _set(&config, &dir, &version) {
        Ok(()) => {}
        Err(text) => {
            writeln!(&mut stderr(), "Error: {}", text).ok();
            exit(1);
        }
    }
}
