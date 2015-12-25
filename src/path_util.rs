use std::path::{Path, PathBuf};

pub trait RelativeExt {
    fn rel_to<'x>(&'x self, other: &'x Path) -> Option<&'x Path>;
}

impl RelativeExt for PathBuf {
    fn rel_to<'x>(&'x self, other: &'x Path) -> Option<&'x Path> {
        let mut iter = self.components();
        for (their, my) in other.components().zip(iter.by_ref()) {
            if my != their {
                return None;
            }
        }
        Some(iter.as_path())
    }
}
