use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub use regex::{Regex, Error, Captures};


lazy_static! {
    static ref CACHE: Mutex<HashMap<String, Arc<Regex>>> =
        Mutex::new(HashMap::new());
}

pub fn compile<'x, S: AsRef<str>>(x: S) -> Result<Arc<Regex>, Error> {
    let s = x.as_ref();
    if let Some(r) = CACHE.lock().unwrap().get(s) {
        return Ok(r.clone());
    }
    Regex::new(s).map(|compiled| {
        let arc = Arc::new(compiled);
        CACHE.lock().unwrap().insert(s.to_string(), arc.clone());
        arc
    })
}
