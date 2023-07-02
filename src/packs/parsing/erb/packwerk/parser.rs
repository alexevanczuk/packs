use regex::Regex;

use crate::packs::{parsing::Range, ProcessedFile, UnresolvedReference};
use std::{fs, path::Path};

use crate::packs::parsing::ruby::packwerk::parser::process_from_contents as process_from_ruby_contents;

pub(crate) fn process_from_path(path: &Path) -> ProcessedFile {
    let contents = fs::read_to_string(path).unwrap_or_else(|_| {
        panic!("Failed to read contents of {}", path.to_string_lossy())
    });

    process_from_contents(contents, path)
}

pub(crate) fn process_from_contents(
    contents: String,
    path: &Path,
) -> ProcessedFile {
    let regex_pattern = r#"(?s)<%=?-?\s*(.*?)\s*-?%>"#;
    let regex = Regex::new(regex_pattern).unwrap();

    let extracted_contents: Vec<&str> = regex
        .captures_iter(&contents)
        .map(|capture| capture.get(1).unwrap().as_str())
        .collect();

    let ruby_contents = extracted_contents.join("\n");
    let processed_file = process_from_ruby_contents(ruby_contents, path);
    let references = processed_file.unresolved_references;
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

    ProcessedFile {
        absolute_path: path.to_path_buf(),
        unresolved_references: references_without_range,
        definitions: vec![],
    }
}
