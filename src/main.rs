extern crate quire;
extern crate argparse;
extern crate tar;
extern crate scan_dir;
extern crate rustc_serialize;


mod config;
mod pack;

use std::str::FromStr;

use argparse::{ArgumentParser, Store, Print, List};


enum Action {
    Help,
    Pack,
}

impl FromStr for Action {
    type Err = ();
    fn from_str(value: &str) -> Result<Action, ()> {
        match value {
            "help" => Ok(Action::Help),
            "pack" => Ok(Action::Pack),
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
                "Command to run. Currently only `pack` is supported");
        ap.refer(&mut args)
            .add_argument("arugments", List,
                "Arguments for the command");
        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }
    match command {
        Action::Help => {
            println!("Usage:");
            println!("    tin pack [options]");
        }
        Action::Pack => {
            args.insert(0, "tin pack".to_string());
            pack::pack(args);
        }
    }
}
