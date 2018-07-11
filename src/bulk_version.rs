use quire::ast::{Ast, Tag};
use quire::{Error as QuireError, ErrorCollector};
use quire::validate as V;

use version::Version;

#[derive(Debug)]
pub struct MinimumVersion(pub Version<&'static str>);

impl V::Validator for MinimumVersion {
    fn default(&self, _pos: V::Pos) -> Option<Ast> {
        None
    }
    fn validate(&self, ast: Ast, errors: &ErrorCollector) -> Ast {
        let (pos, kind, val) = match ast {
            Ast::Scalar(pos, _, kind, min_version) => {
                if Version(&min_version[..]) > self.0 {
                    errors.add_error(QuireError::validation_error(&pos,
                        format!("This package configure requires bulk \
                                 of at least {:?}", min_version)));
                }
                (pos, kind, min_version)
            }
            ast => {
                errors.add_error(QuireError::validation_error(&ast.pos(),
                    format!("Value of bulk-version must be scalar")));
                return ast;
            }
        };
        return Ast::Scalar(pos, Tag::NonSpecific, kind, val)
    }
}
