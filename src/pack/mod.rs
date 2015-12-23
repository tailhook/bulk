mod metadata;

use std::io::{stdout, stderr, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

use argparse::{ArgumentParser, Parse};
use config::Config;
use self::metadata::populate;


fn _pack(config: &Path) -> Result<(), String> {
    let cfg = try!(Config::parse_file(config));
    let meta = try!(populate(&cfg));
    println!("Mdata {:#?}", meta);
    Ok(())
}


pub fn pack(args: Vec<String>) {
    let mut config = PathBuf::from("package.yaml");
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut config)
            .add_option(&["-c", "--config"], Parse,
                "Package configuration file");
        ap.stop_on_first_argument(true);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => exit(x),
        }
    }

    match _pack(&config) {
        Ok(()) => {}
        Err(text) => {
            writeln!(&mut stderr(), "{}", text).ok();
            exit(1);
        }
    }
}
