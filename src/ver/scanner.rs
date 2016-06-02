use std::io::{self, BufRead};
use std::sync::Arc;

use re;
use config::VersionHolder;


#[derive(Clone)]
pub struct Scanner {
    pub block_start: Option<Arc<re::Regex>>,
    pub block_end: Option<Arc<re::Regex>>,
    pub regex: Arc<re::Regex>,
}

pub struct Lines<B: BufRead>(usize, B);

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

impl<B:BufRead> Lines<B> {
    pub fn iter(file: B) -> Lines<B> {
        Lines(0, file)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Prefix,
    Body,
    Suffix,
}

pub struct Iter<'a> {
    scanner: &'a Scanner,
    state: State,
    error: Option<Error>,
}

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        EmptyVersion(line_no: usize) {
            display("{}: empty version string captured", line_no)
            description("empty version")
        }
        NoCapture(line_no: usize) {
            display("{}: no capture in regex, probably bad regex", line_no)
            description("no capture in regex, probably bad regex")
        }
    }
}

impl<'a> Iter<'a> {

    pub fn error(self) -> Result<(), Error> {
        self.error.map(Err).unwrap_or(Ok(()))
    }

    pub fn line(&mut self, line_no: usize, line: &str)
        -> Option<(usize, usize)>
    {
        use self::State::*;
        use self::Error::*;
        if self.error.is_some() {
            return None;
        }
        match self.state {
            Prefix => {
                if self.scanner.block_start.as_ref().unwrap().is_match(line) {
                    self.state = Body;
                }
                None
            }
            Body => match self.scanner.regex.captures(line) {
                Some(cpt) => match cpt.pos(1) {
                    Some((x, y)) if x == y => {
                        self.error = Some(EmptyVersion(line_no));
                        return None;
                    }
                    Some(x) => Some(x),
                    None => {
                        self.error = Some(NoCapture(line_no));
                        return None;
                    }
                },
                None => {
                    match self.scanner.block_end {
                        Some(ref end_re) if end_re.is_match(line) => {
                            self.state = Suffix;
                        }
                        _ => {}
                    }
                    None
                }
            },
            Suffix => None,
        }
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
    pub fn start(&self) -> Iter {
        use self::State::*;
        Iter {
            scanner: self,
            state: if self.block_start.is_some()
                { Prefix } else { Body },
            error: None,
        }
    }
}
