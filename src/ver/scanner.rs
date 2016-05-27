use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::iter;
use std::path::Path;
use std::sync::Arc;

use re;
use config::VersionHolder;


pub struct Scanner {
    pub block_start: Option<Arc<re::Regex>>,
    pub block_end: Option<Arc<re::Regex>>,
    pub regex: Arc<re::Regex>,
}

struct Lines<B: BufRead>(usize, B);

impl<B: BufRead> Iterator for Lines<B> {
    type Item = io::Result<(usize, String)>;
    fn next(&mut self) -> Option<io::Result<(usize, String)>> {
        let Lines(ref mut lineno, ref mut file) = *self;
        let mut s = String::with_capacity(200);
        match file.read_line(&mut s) {
            Ok(0) => None,
            Ok(_) => {
                *lineno += 1;
                Some(Ok((*lineno, s)))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

fn _find_block_start<B: BufRead, F>(lines: &mut Lines<B>,
    bstart: &Option<Arc<re::Regex>>, fun: &mut F)
    -> Result<bool, io::Error>
    where F: FnMut(usize, &str, Option<re::Captures>) -> bool
{
    if let Some(ref re) = *bstart {
        for item in lines {
            let (n, line) = try!(item);
            if !fun(n, &line, None) {
                return Ok(false);
            }
            if re.is_match(&line) {
                return Ok(true);
            }
        }
        Ok(false)
    } else {
        Ok(true)
    }
}

impl Scanner {
    pub fn new(cfg: &VersionHolder) -> Result<Scanner, re::Error> {
        Ok(Scanner {
            block_start: if let Some(ref regex) = cfg.block_start {
                             Some(try!(re::compile(regex)))
                         } else { None },
            block_end: if let Some(ref regex) = cfg.block_end {
                           Some(try!(re::compile(regex)))
                       } else { None },
            regex: try!(re::compile(&cfg.regex)),
        })
    }
    pub fn scan_file<F, P: AsRef<Path>>(&self, path: P, mut fun: F)
        -> Result<(), io::Error>
        where F: FnMut(usize, &str, Option<re::Captures>) -> bool,
    {
        match File::open(path) {
            Ok(x) => {
                self.scan(&mut Lines(0, BufReader::new(x)), fun)
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    }
    fn scan<B, F>(&self, lines: &mut Lines<B>, mut fun: F)
        -> Result<(), io::Error>
        where F: FnMut(usize, &str, Option<re::Captures>) -> bool,
              B: BufRead,
    {
        if !try!(_find_block_start(lines, &self.block_start, &mut fun)) {
            return Ok(());
        }
        let block_re = self.block_end.as_ref();
        for item in lines.by_ref() {
            let (lineno, line) = try!(item);
            if !fun(lineno, &line, self.regex.captures(&line)) {
                return Ok(());
            }
            if block_re.as_ref().map(|re| re.is_match(&line)).unwrap_or(false) {
                break;
            }
        }
        for item in lines {
            let (lineno, line) = try!(item);
            if !fun(lineno, &line, None) {
                return Ok(());
            }
        }
        Ok(())
    }
}
