extern crate quire;
extern crate argparse;
extern crate tar;
extern crate scan_dir;
extern crate rustc_serialize;
extern crate sha2;
extern crate time;
extern crate flate2;
extern crate regex;
extern crate unicase;
extern crate env_logger;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;


mod expand;
mod config;
mod pack;
mod repo;
mod deb_ext;
mod hash_file;

use std::str::FromStr;

use argparse::{ArgumentParser, Store, Print, List};


enum Action {
    Help,
    Pack,
    RepoAdd,
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
            "Show version of tin and exit");
        ap.refer(&mut command)
            .add_argument("command", Store,
                "Command to run. Supported commands: pack, repo-add");
        ap.refer(&mut args)
            .add_argument("arguments", List,
                "Arguments for the command");
        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }
    env_logger::init().expect("init logging system");
    match command {
        Action::Help => {
            println!("Usage:");
            println!("    tin {{pack,repo-add}} [options]");
        }
        Action::Pack => {
            args.insert(0, "tin pack".to_string());
            pack::pack(args);
        }
        Action::RepoAdd => {
            args.insert(0, "tin repo-add".to_string());
            repo::repo_add(args);
        }
    }
}
