mod metadata;
mod ar;
mod deb;

use std::io::{self, stdout, stderr};
use std::path::PathBuf;
use std::process::exit;

use argparse::{ArgumentParser, Parse, Collect};

use repo::metadata::gather_metadata;


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

    let packages = packages.iter().map(gather_metadata)
        .collect::<Result<Vec<_>, io::Error>>();
    println!("Packages {:#?}",  packages);
}
