mod metadata;
mod ar;
mod deb;
mod debian;

use std::io::{self, stdout, stderr, Write};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::process::exit;

use regex::Regex;
use argparse::{ArgumentParser, Parse, Collect};

use config::{Config, RepositoryType};
use repo::metadata::gather_metadata;


fn _repo_add(config: &Path, packages: &Vec<String>, dir: &Path)
    -> Result<(), Box<Error>>
{
    let packages = try!(packages.iter().map(gather_metadata)
        .collect::<Result<Vec<_>, _>>());
    debug!("Packages read {:#?}", packages);
    let cfg = try!(Config::parse_file(&config));
    let mut debian = debian::Repository::new(dir);

    for repo in &cfg.repositories {
        let version_re = match repo.match_version {
            Some(ref re) => Some(try!(Regex::new(re))),
            None => None,
        };
        let matching = packages.iter()
            .filter(|p| {
                repo.architecture.as_ref().map(|x| x == &p.arch)
                .unwrap_or(true) &&
                version_re.as_ref().map(|x| x.is_match(&p.version))
                .unwrap_or(true)
            })
            .collect::<Vec<_>>();
        if matching.len() > 0 {
            match (repo.kind, &repo.suite, &repo.component) {
                (RepositoryType::debian, &Some(ref suite), &Some(ref comp))
                => {
                    let cur = try!(debian.open(suite, comp));
                    for p in matching {
                        cur.add_package(p);
                    }
                    //cur.retent(repo.keep_releases);
                }
                (RepositoryType::debian, _, _) => {
                    return Err("Debian repository requires suite and \
                               component to be specified".into());

                }
            }
        }
    }
    println!("{:#?}", debian);
    return Err("not implemented".into());
}


pub fn repo_add(args: Vec<String>) {
    let mut config = PathBuf::from("package.yaml");
    let mut repo_dir = PathBuf::new();
    let mut packages = Vec::<String>::new();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut config)
            .add_option(&["-c", "--config"], Parse,
                "Package configuration file");
        ap.refer(&mut repo_dir)
            .add_option(&["-D", "--repository-base"], Parse,
                "Directory where repositories are stored");
        ap.refer(&mut packages)
            .add_argument("packages", Collect,
                "Package file names to add");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => exit(x),
        }
    }

    match _repo_add(&config, &packages, &repo_dir) {
        Ok(()) => {}
        Err(err) => {
            writeln!(&mut stderr(), "{}", err).ok();
            exit(1);
        }
    }
}
