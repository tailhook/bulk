use std::default::Default;
use std::path::Path;

use quire::validate::{Sequence, Structure, Enum, Nothing, Numeric};
use quire::parse_config;

use expand::Value;

#[derive(RustcDecodable, Clone, Debug)]
pub struct Metadata {
    pub name: String,
    pub architecture: String,
    pub short_description: String,
    pub long_description: String,
    pub version: String,
}

#[allow(non_camel_case_types)]
#[derive(RustcDecodable, Clone, Copy, Debug)]
pub enum RepositoryType {
    debian,
}

#[derive(RustcDecodable, Clone, Debug)]
pub struct Repository {
    pub kind: RepositoryType,
    pub suite: Option<String>,
    pub component: Option<String>,
    pub architecture: Option<String>,
    pub keep_releases: Option<usize>,
    pub match_version: Option<String>,
}

#[derive(RustcDecodable, Clone, Debug)]
pub struct Config {
    pub metadata: Metadata,
    pub repositories: Vec<Repository>,
}

impl Config {
    fn validator<'x>() -> Structure<'x> {
        Structure::new()
        .member("metadata", Structure::new()
            .member("name", Value(false))
            .member("architecture", Value(false))
            .member("short_description", Value(false))
            .member("long_description", Value(false))
            .member("version", Value(false)))
        .member("repositories", Sequence::new(Structure::new()
            .member("kind", Enum::new().allow_plain()
                .option("debian", Nothing)
            )
            .member("suite", Value(true))
            .member("component", Value(true))
            .member("architecture", Value(true))
            .member("keep_releases", Numeric::new().optional())
            .member("match_version", Value(true))))
    }
    pub fn parse_file(p: &Path) -> Result<Config, String> {
        parse_config(p, &Config::validator(), Default::default())
    }
}
