use std::io::{self, Read, BufRead, BufReader};
use std::mem::replace;
use std::collections::HashMap;

use unicase::UniCase;


fn error(text: &'static str) -> io::Error {
    return io::Error::new(io::ErrorKind::Other, text);
}


pub fn parse_control<R: Read>(r: R)
    -> io::Result<Vec<HashMap<UniCase<String>, String>>>
{
    let src = BufReader::new(r);
    let mut res = Vec::new();
    let mut current_hash = HashMap::new();
    let mut buf = None::<(String, String)>;
    for line in src.lines() {
        let line = try!(line);
        if line.len() == 0 {
            if let Some((key, val)) = buf.take() {
                current_hash.insert(UniCase(key), val);
            }
            if current_hash.len() > 0 {
                res.push(replace(&mut current_hash, HashMap::new()));
            }
        } else if line.starts_with(' ') {
            if let Some((_, ref mut val)) = buf {
                val.push_str("\n");
                if line != " ." {
                    val.push_str(&line[1..]);
                }
            } else {
                return Err(error("Bad format of debian control"));
            }
        } else {
            if let Some((key, val)) = buf.take() {
                current_hash.insert(UniCase(key), val);
            }
            let mut pair = line.splitn(2, ':');
            match (pair.next(), pair.next()) {
                (Some(k), Some(v)) => {
                    buf = Some((k.to_string(), v.trim().to_string()));
                }
                _ => return Err(error("Bad format of debian control")),
            }
        }
    }
    if let Some((key, val)) = buf.take() {
        current_hash.insert(UniCase(key), val);
    }
    if current_hash.len() > 0 {
        res.push(current_hash);
    }
    return Ok(res);
}
