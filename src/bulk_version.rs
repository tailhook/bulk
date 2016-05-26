use quire::ast::{Ast, Tag};
use quire::sky::{Error as QuireError};
use quire::validate as V;

use version::Version;

pub struct MinimumVersion(pub Version<&'static str>);

impl V::Validator for MinimumVersion {
    fn default(&self, _pos: V::Pos) -> Option<Ast> {
        None
    }
    fn validate(&self, ast: Ast) -> (Ast, Vec<QuireError>) {
        let mut warnings = vec!();
        let (pos, kind, val) = match ast {
            Ast::Scalar(pos, _, kind, min_version) => {
                if Version(&min_version[..]) > self.0 {
                    warnings.push(QuireError::validation_error(&pos,
                        format!("This package configure requires bulk \
                                 of at least {:?}", min_version)));
                }
                (pos, kind, min_version)
            }
            ast => {
                warnings.push(QuireError::validation_error(&ast.pos(),
                    format!("Value of bulk-version must be scalar")));
                return (ast, warnings);
            }
        };
        (Ast::Scalar(pos, Tag::NonSpecific, kind, val), warnings)
    }
}
