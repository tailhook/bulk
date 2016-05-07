use std::sync::Mutex;
use std::process::Command;
use std::collections::HashMap;

use regex::{Regex, Captures};
use quire::validate::{Validator, Pos};
use quire::ast::{Ast, Tag, NullKind};
use quire::sky::Error;

lazy_static! {
    static ref SHELL_REGEX: Regex =
        Regex::new(r#"\$\(([^)]+)\)"#).expect("regex compiles");
    static ref CACHE: Mutex<HashMap<String, String>> =
        Mutex::new(HashMap::new());
}

pub struct Value(pub bool);

impl Validator for Value {
    fn default(&self, pos: Pos) -> Option<Ast> {
        Some(Ast::Null(pos.clone(), Tag::NonSpecific, NullKind::Implicit))
    }
    fn validate(&self, ast: Ast) -> (Ast, Vec<Error>) {
        let mut warnings = vec!();
        let (pos, kind, val) = match ast {
            Ast::Scalar(pos, _, kind, string) => {
                let string = SHELL_REGEX.replace(&string, |caps: &Captures| {
                    let expr = caps.at(1).unwrap();
                    let mut cache = CACHE.lock().unwrap();
                    if let Some(value) = cache.get(expr) {
                        return value.clone().into();
                    }
                    match run_script(expr) {
                        Ok(x) => {
                            cache.insert(expr.to_string(), x.clone());
                            x.into()
                        }
                        Err(e) => {
                            warnings.push(Error::validation_error(&pos,
                                format!("Expansion error: {}", e)));
                            "".into()
                        }
                    }
                });
                (pos, kind, string)
            }
            Ast::Null(_, _, _) => {
                if !self.0 {
                    warnings.push(Error::validation_error(&ast.pos(),
                        format!("Value is required")));
                }
                return (ast, warnings);
            }
            ast => {
                warnings.push(Error::validation_error(&ast.pos(),
                    format!("Value must be scalar")));
                return (ast, warnings);
            }
        };
        return (Ast::Scalar(pos, Tag::NonSpecific, kind, val), warnings);
    }
}


fn run_script(cmdtext: &str) -> Result<String, String> {
    let mut cmd = Command::new("/bin/sh");
    cmd.arg("-c");
    cmd.arg(cmdtext);
    match cmd.output() {
        Err(e) => Err(format!("Error executing command {:?}: {}", cmd, e)),
        Ok(output) => {
            if output.status.success() {
                String::from_utf8(output.stdout)
                .map(|x| x.trim().to_string())
                .map_err(|e| format!("Error executing command {:?}: \
                    error decoding output: {}", cmd, e))
            } else {
                 Err(format!("Error executing command {:?}: {:?}",
                    cmd, output.status))
            }
        }
    }
}
