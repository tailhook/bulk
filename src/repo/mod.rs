mod metadata;
mod ar;
mod deb;
mod debian;

use std::io::{stdout, stderr, Write};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::process::exit;

use regex::Regex;
use argparse::{ArgumentParser, Parse, Collect, StoreConst};

use config::{Config, RepositoryType};
use repo::metadata::gather_metadata;


fn _repo_add(config: &Path, packages: &Vec<String>, dir: &Path,
    on_conflict: debian::ConflictResolution)
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
        let skip_re = match repo.skip_version {
            Some(ref re) => Some(try!(Regex::new(re))),
            None => None,
        };
        let matching = packages.iter()
            .filter(|p| {
                version_re.as_ref().map(|x| x.is_match(&p.version))
                .unwrap_or(true) &&
                !skip_re.as_ref().map(|x| x.is_match(&p.version))
                .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        if matching.len() > 0 {
            match (repo.kind, &repo.suite, &repo.component) {
                (RepositoryType::debian, &Some(ref suite), &Some(ref comp))
                => {
                    for p in matching {
                        try!(try!(debian.open(suite, comp, &p.arch))
                            .add_package(p, on_conflict));
                        if repo.add_empty_i386_repo && p.arch != "i386" {
                            try!(debian.open(suite, comp, "i386"));
                        }
                    }
                }
                (RepositoryType::debian, _, _) => {
                    return Err("Debian repository requires suite and \
                               component to be specified".into());

                }
            }
        }
    }
    for repo in cfg.repositories {
        match (repo.kind, &repo.suite, &repo.component) {
            (RepositoryType::debian, &Some(ref suite), &Some(ref comp)) => {
                if let Some(limit) = repo.keep_releases {
                    debian.trim(suite, comp, limit);
                }
            }
            _ => unreachable!(),
        }
    }
    try!(debian.write());
    // TODO(tailhook) remove removed files
    Ok(())
}


pub fn repo_add(args: Vec<String>) {
    let mut config = PathBuf::from("bulk.yaml");
    let mut repo_dir = PathBuf::new();
    let mut packages = Vec::<String>::new();
    let mut conflict = debian::ConflictResolution::Error;
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut config)
            .add_option(&["-c", "--config"], Parse,
                "Package configuration file");
        ap.refer(&mut repo_dir)
            .add_option(&["-D", "--repository-base"], Parse,
                "Directory where repositories are stored");
        ap.refer(&mut conflict)
            .add_option(&["--skip-existing"],
                StoreConst(debian::ConflictResolution::Keep),
                "Skip package if it's already in the repository")
            .add_option(&["--replace-existing"],
                StoreConst(debian::ConflictResolution::Replace),
                "Replace package if it's already in the repository");
        ap.refer(&mut packages)
            .add_argument("packages", Collect,
                "Package file names to add");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => exit(x),
        }
    }

    match _repo_add(&config, &packages, &repo_dir, conflict) {
        Ok(()) => {}
        Err(err) => {
            writeln!(&mut stderr(), "Error: {}", err).ok();
            exit(1);
        }
    }
}
