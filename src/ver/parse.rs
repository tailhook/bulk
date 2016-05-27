use std::io::{self, BufRead, BufReader};
use std::iter;
use std::fs::{File};
use std::path::{Path, PathBuf};

use quick_error::ResultExt;

use config::VersionHolder;
use version::Version;
use re;


type Lines<B> = iter::Enumerate<io::Lines<B>>;
type Location = (PathBuf, usize, Version<String>);

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Regex(re: String, err: re::Error) {
            display("can't compile regex {:?}: {}", re, err)
            description("can't compile regular expression")
            context(re: AsRef<str>, err: re::Error)
                -> (re.as_ref().to_string(), err)
        }
        NoCapture {
            description("version regex doesn't container capture group")
        }
        CapturedEmptyVersion(line: usize) {
            display("{}: captured empty version number", line)
            description("captured empty version number")
        }
        Io(err: io::Error) {
            display("{}", err)
            description("IO error")
            from()
        }
    }
}

fn _find_block_start<B: BufRead>(lines: &mut Lines<B>, cfg: &VersionHolder)
    -> Result<bool, Error>
{
    if let Some(ref x) = cfg.block_start {
        let re = try!(re::compile(x).context(x));
        for (n, line) in lines {
            let line = try!(line);
            if re.is_match(&line) {
                return Ok(true);
            }
        }
        Ok(false)
    } else {
        Ok(true)
    }
}

pub fn get_first<P: AsRef<Path>>(file: P, cfg: &VersionHolder)
    -> Result<Option<Version<String>>, Error>
{
    Ok(try!(get_all(file, cfg)).pop().map(|(_, v)| v))
}

pub fn get_all<P: AsRef<Path>>(file: P, cfg: &VersionHolder)
    -> Result<Vec<(usize, Version<String>)>, Error>
{
    let mut f = match File::open(file) {
        Ok(x) => BufReader::new(x),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(e) => return Err(e.into()),
    };
    let mut buf = Vec::new();
    try!(scan_versions(&mut f.lines().enumerate(), cfg, &mut buf));
    Ok(buf)
}

fn scan_versions<B: BufRead>(lines: &mut Lines<B>, cfg: &VersionHolder,
    res: &mut Vec<(usize, Version<String>)>)
    -> Result<(), Error>
{
    if !try!(_find_block_start(lines, cfg)) {
        return Ok(());
    }
    let version_re = try!(re::compile(&cfg.regex).context(&cfg.regex));
    let block_re = if let Some(ref x) = cfg.block_end {
        Some(try!(re::compile(x).context(x)))
    } else {
        None
    };
    for (lineno, line) in lines {
        let line = try!(line);
        if let Some(capt) = version_re.captures(&line) {
            match capt.at(1) {
                Some("") => {
                    return Err(Error::CapturedEmptyVersion(lineno));
                }
                Some(x) => {
                    res.push((lineno, Version(x.into())));
                }
                None => {
                    return Err(Error::NoCapture);
                }
            }
        }
        if block_re.as_ref().map(|re| re.is_match(&line)).unwrap_or(false) {
            break;
        }
    }
    Ok(())
}
