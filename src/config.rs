use std::default::Default;
use std::path::Path;

use quire::validate::*;
use quire::parse_config;


#[derive(RustcDecodable, Clone, Debug)]
pub struct Metadata {
    pub name: Option<String>,
    pub short_description: Option<String>,
    pub long_description: Option<String>,
    pub version: Option<String>,
}

#[derive(RustcDecodable, Clone, Debug)]
pub struct Scripts {
    pub name: Option<String>,
    pub short_description: Option<String>,
    pub long_description: Option<String>,
    pub version: Option<String>,
}

#[derive(RustcDecodable, Clone, Debug)]
pub struct Config {
    pub metadata: Metadata,
    pub scripts: Scripts,
}

impl Config {
    fn validator<'x>() -> Structure<'x> {
        Structure::new()
        .member("metadata", Structure::new()
            .member("name", Scalar::new().optional())
            .member("short_description", Scalar::new().optional())
            .member("long_description", Scalar::new().optional())
            .member("version", Scalar::new().optional()))
        .member("scripts", Structure::new()
            .member("name", Scalar::new().optional())
            .member("short_description", Scalar::new().optional())
            .member("long_description", Scalar::new().optional())
            .member("version", Scalar::new().optional()))
    }
    pub fn parse_file(p: &Path) -> Result<Config, String> {
        parse_config(p, &Config::validator(), Default::default())
    }
}
