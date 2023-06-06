use crate::packs::{Range, UnresolvedReference};
use std::{fs, path::PathBuf};

use crate::packs::parser::ruby::packwerk::extractor::extract_from_contents as extract_from_ruby_contents;

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
    let start_tag = "<%=";
    let end_tag = "%>";

    let mut extracted_contents: Vec<&str> = Vec::new();

    let mut search_start_index = 0;

    while let Some(start_index) = contents[search_start_index..].find(start_tag)
    {
        let real_start_index = search_start_index + start_index;
        let end_index = match contents[real_start_index..].find(end_tag) {
            Some(index) => real_start_index + index,
            None => break,
        };

        let extracted_content =
            &contents[(real_start_index + start_tag.len())..end_index];
        extracted_contents.push(extracted_content);

        search_start_index = end_index + end_tag.len();
    }

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
