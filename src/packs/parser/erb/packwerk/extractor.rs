use regex::Regex;

use crate::packs::{Range, UnresolvedReference};
use std::{fs, path::Path};

use crate::packs::parser::ruby::packwerk::extractor::extract_from_contents as extract_from_ruby_contents;

pub(crate) fn extract_from_path(path: &Path) -> Vec<UnresolvedReference> {
    let contents = fs::read_to_string(path).unwrap_or_else(|_| {
        panic!("Failed to read contents of {}", path.to_string_lossy())
    });

    extract_from_contents(contents)
}

pub(crate) fn extract_from_contents(
    contents: String,
) -> Vec<UnresolvedReference> {
    let regex_pattern = r#"(?s)<%=?-?\s*(.*?)\s*-?%>"#;
    let regex = Regex::new(regex_pattern).unwrap();

    let extracted_contents: Vec<&str> = regex
        .captures_iter(&contents)
        .map(|capture| capture.get(1).unwrap().as_str())
        .collect();

    let ruby_contents = extracted_contents.join("\n");
    let references = extract_from_ruby_contents(ruby_contents);
    // let references_without_range = references
    let references_without_range = references
        .iter()
        .map(|r| UnresolvedReference {
            // Source maps are not yet supported for ERB, since we just turn it into Ruby code
            // that doesn't necessarily map up to the original.
            // We need to add extra logic to support source maps (or use a proper parsing library).
            location: Range::default(),
            ..r.clone()
        })
        .collect();
    references_without_range
}
