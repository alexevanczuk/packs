use crate::packs::UnresolvedReference;
use std::{fs, path::PathBuf};

#[allow(dead_code)]
pub(crate) fn extract_from_path(path: &PathBuf) -> Vec<UnresolvedReference> {
    let contents = fs::read_to_string(path).unwrap_or_else(|_| {
        panic!("Failed to read contents of {}", path.to_string_lossy())
    });

    extract_from_contents(contents)
}

#[allow(unused_variables)]
pub(crate) fn extract_from_contents(
    contents: String,
) -> Vec<UnresolvedReference> {
    todo!();
}
