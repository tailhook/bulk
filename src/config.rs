use std::path::{Path, PathBuf};

use regex::Regex;
use quire::validate::{Sequence, Structure, Enum, Nothing, Numeric, Scalar};
use quire::{parse_config, Options};

use version::Version;
use bulk_version::MinimumVersion;


#[derive(Deserialize, Clone, Debug)]
pub struct Metadata {
    pub name: String,
    pub short_description: String,
    pub long_description: String,
    pub depends: Option<String>,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all="kebab-case")]
pub enum RepositoryType {
    Debian,
    HtmlLinks,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Repository {
    pub kind: RepositoryType,
    pub keep_releases: Option<usize>,
    #[serde(with="::serde_regex")]
    pub match_version: Option<Regex>,
    #[serde(with="::serde_regex")]
    pub match_filename: Option<Regex>,
    #[serde(with="::serde_regex")]
    pub skip_version: Option<Regex>,

    // debian
    pub suite: Option<String>,
    pub component: Option<String>,
    // This hack is needed for old ubuntu which want to download indexes for
    // i386 packages even on amd64 even if you will never try to install them
    pub add_empty_i386_repo: bool,
    // html-links
    pub files: Option<PathBuf>,
    pub index: Option<PathBuf>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub minimum_bulk: Version<String>,
    pub metadata: Option<Metadata>,
    pub repositories: Vec<Repository>,
    pub versions: Vec<VersionHolder>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct VersionHolder {
    pub block_start: Option<String>,
    pub block_end: Option<String>,
    pub multiple_blocks: bool,
    pub file: Option<PathBuf>,
    pub files: Vec<PathBuf>,
    pub regex: String,
    pub partial_version: Option<String>,
}

impl Config {
    fn validator<'x>() -> Structure<'x> {
        Structure::new()
        .member("minimum_bulk", MinimumVersion(
            Version(env!("CARGO_PKG_VERSION"))))
        .member("metadata", Structure::new().optional()
            .member("name", Scalar::new())
            .member("short_description", Scalar::new())
            .member("long_description", Scalar::new())
            .member("depends", Scalar::new().optional()))
        .member("repositories", Sequence::new(Structure::new()
            .member("kind", Enum::new().allow_plain()
                .option("debian", Nothing)
                .option("html-links", Nothing)
            )
            .member("keep_releases", Numeric::new().optional())
            .member("match_version", Scalar::new().optional())
            .member("match_filename", Scalar::new().optional())
            .member("skip_version", Scalar::new().optional())
            // debian
            .member("suite", Scalar::new().optional())
            .member("component", Scalar::new().optional())
            .member("add_empty_i386_repo", Scalar::new().default(false))
            // html links
            .member("files", Scalar::new().optional())
            .member("index", Scalar::new().optional())))
        .member("versions", Sequence::new(Structure::new()
            .member("block_start", Scalar::new().optional())
            .member("block_end", Scalar::new().optional())
            .member("multiple_blocks", Scalar::new().default(false))
            .member("file", Scalar::new().optional())
            .member("files", Sequence::new(Scalar::new()))
            .member("regex", Scalar::new())
            .member("partial_version", Scalar::new().optional())))
    }
    pub fn parse_file(p: &Path) -> Result<Config, String> {
        Ok(parse_config(p, &Config::validator(), &Options::default())
            .map_err(|e| e.to_string())?)
    }
}
