use std::io::{self, BufRead, BufReader};
use std::fs::{File};
use std::path::Path;

use quick_error::ResultExt;

use config::VersionHolder;
use version::Version;
use re;


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
        CapturedEmptyVersion(line: String) {
            display("captured empty version number in {:?}", line)
            description("captured empty version number")
        }
        Io(err: io::Error) {
            display("{}", err)
            description("IO error")
            from()
        }
    }
}


pub fn _find_block_start<B: BufRead>(file: &mut B, cfg: &VersionHolder)
    -> Result<bool, Error>
{
    if let Some(ref x) = cfg.block_start {
        let re = try!(re::compile(x).context(x));
        for line in file.lines() {
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
    let mut f = match File::open(file) {
        Ok(x) => BufReader::new(x),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    if !try!(_find_block_start(&mut f, cfg)) {
        return Ok(None);
    }
    let version_re = try!(re::compile(&cfg.regex).context(&cfg.regex));
    let block_re = if let Some(ref x) = cfg.block_end {
        Some(try!(re::compile(x).context(x)))
    } else {
        None
    };
    for line in f.lines() {
        let line = try!(line);
        if let Some(capt) = version_re.captures(&line) {
            match capt.at(1) {
                Some("") => {
                    return Err(Error::CapturedEmptyVersion(line.clone()));
                }
                Some(x) => return Ok(Some(Version(x.into()))),
                None => {
                    return Err(Error::NoCapture);
                }
            }
        }
        if block_re.as_ref().map(|re| re.is_match(&line)).unwrap_or(false) {
            break;
        }
    }
    Ok(None)
}
