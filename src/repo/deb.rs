use std::io::{self, Read, BufRead, BufReader};
use std::collections::HashMap;

use unicase::UniCase;


fn error(text: &'static str) -> io::Error {
    return io::Error::new(io::ErrorKind::Other, text);
}


pub fn parse_control<R: Read>(r: R)
    -> io::Result<HashMap<UniCase<String>, String>>
{
    let src = BufReader::new(r);
    let mut result = HashMap::new();
    let mut buf = None::<(String, String)>;
    for line in src.lines() {
        let line = try!(line);
        if line.len() == 0 || line.starts_with(' ') {
            if let Some((_, ref mut val)) = buf {
                val.push_str("\n");
                val.push_str(&line.trim());
            } else {
                return Err(error("Bad format of debian control"));
            }
        } else {
            if let Some((key, val)) = buf.take() {
                result.insert(UniCase(key), val);
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
        result.insert(UniCase(key), val);
    }
    return Ok(result);
}
