extern crate argparse;
extern crate env_logger;
extern crate libflate;
extern crate git2;
extern crate quire;
extern crate regex;
extern crate serde;
extern crate scan_dir;
extern crate sha2;
extern crate tar;
extern crate tempfile;
extern crate time;
extern crate unicase;
#[macro_use] extern crate log;
#[macro_use] extern crate matches;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate serde_derive;


mod config;
mod deb_ext;
mod hash_file;
mod version;
mod bulk_version;
mod re;

mod repo;
mod pack;
mod ver;

use std::str::FromStr;

use argparse::{ArgumentParser, Store, Print, List};


enum Action {
    Help,
    Pack,
    RepoAdd,
    GetVersion,
    SetVersion,
    IncrVersion,
    CheckVersion,
    WithVersion,
    WithGitVersion,
}

impl FromStr for Action {
    type Err = ();
    fn from_str(value: &str) -> Result<Action, ()> {
        match value {
            "help" => Ok(Action::Help),

            "pack" => Ok(Action::Pack),

            "repo-add" => Ok(Action::RepoAdd),
            "repo_add" => Ok(Action::RepoAdd),
            "repoadd" => Ok(Action::RepoAdd),
            "radd" => Ok(Action::RepoAdd),
            "add-to-repo" => Ok(Action::RepoAdd),
            "add_to_repo" => Ok(Action::RepoAdd),
            "addtorepo" => Ok(Action::RepoAdd),

            "getversion" => Ok(Action::GetVersion),
            "get-version" => Ok(Action::GetVersion),
            "getver" => Ok(Action::GetVersion),
            "get-ver" => Ok(Action::GetVersion),
            "ver-get" => Ok(Action::GetVersion),
            "version-get" => Ok(Action::GetVersion),
            "verget" => Ok(Action::GetVersion),
            "versionget" => Ok(Action::GetVersion),

            "set-version" => Ok(Action::SetVersion),
            "set-ver" => Ok(Action::SetVersion),
            "setversion" => Ok(Action::SetVersion),
            "setver" => Ok(Action::SetVersion),
            "ver-set" => Ok(Action::SetVersion),
            "version-set" => Ok(Action::SetVersion),
            "verset" => Ok(Action::SetVersion),
            "versionset" => Ok(Action::SetVersion),

            "incr-version" => Ok(Action::IncrVersion),
            "inc-version" => Ok(Action::IncrVersion),
            "version-incr" => Ok(Action::IncrVersion),
            "version-inc" => Ok(Action::IncrVersion),
            "incr" => Ok(Action::IncrVersion),
            "bump-version" => Ok(Action::IncrVersion),
            "version-bump" => Ok(Action::IncrVersion),
            "bumpver" => Ok(Action::IncrVersion),
            "bump-ver" => Ok(Action::IncrVersion),
            "bump" => Ok(Action::IncrVersion),

            "check-version" => Ok(Action::CheckVersion),
            "version-check" => Ok(Action::CheckVersion),

            "with-version" => Ok(Action::WithVersion),
            "with-git-version" => Ok(Action::WithGitVersion),

            _ => Err(())
        }
    }
}


fn main() {
    let mut command = Action::Help;
    let mut args = Vec::<String>::new();
    {
        let mut ap = ArgumentParser::new();
        ap.add_option(&["-V", "--version"],
            Print(env!("CARGO_PKG_VERSION").to_string()),
            "Show version of bulk and exit");
        ap.refer(&mut command)
            .add_argument("command", Store, "
                Command to run. Supported commands: \
                pack, repo-add, get-version, set-version, incr-version, \
                check-version, with-version, with-git-version");
        ap.refer(&mut args)
            .add_argument("arguments", List,
                "Arguments for the command");
        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }
    env_logger::init();
    match command {
        Action::Help => {
            println!("Usage:");
            println!("    bulk \
                {{pack,repo-add,get-version,set-verion,\
                  check-version,with-version,git-version}} \
                [options]");
        }
        Action::Pack => {
            args.insert(0, "bulk pack".to_string());
            pack::pack(args);
        }
        Action::RepoAdd => {
            args.insert(0, "bulk repo-add".to_string());
            repo::repo_add(args);
        }
        Action::GetVersion => {
            args.insert(0, "bulk get-version".to_string());
            ver::get_version(args);
        }
        Action::SetVersion => {
            args.insert(0, "bulk set-version".to_string());
            ver::set_version(args);
        }
        Action::IncrVersion => {
            args.insert(0, "bulk incr-version".to_string());
            ver::incr_version(args);
        }
        Action::CheckVersion => {
            args.insert(0, "bulk check-version".to_string());
            ver::check_version(args);
        }
        Action::WithVersion => {
            args.insert(0, "bulk with-version".to_string());
            ver::with_version(args);
        }
        Action::WithGitVersion => {
            args.insert(0, "bulk with-git-version".to_string());
            ver::with_git_version(args);
        }
    }
}
