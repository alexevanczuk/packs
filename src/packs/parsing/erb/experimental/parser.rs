use crate::packs::file_utils::file_read_contents;
use crate::packs::{
    file_utils::convert_erb_to_ruby_without_sourcemaps, parsing::Range,
    Configuration, ProcessedFile, UnresolvedReference,
};
use std::path::Path;

use crate::packs::parsing::ruby::experimental::parser::process_from_contents as process_from_ruby_contents;

pub(crate) fn process_from_path(
    path: &Path,
    configuration: &Configuration,
) -> anyhow::Result<ProcessedFile> {
    let contents = file_read_contents(path, configuration)?;
    Ok(process_from_contents(contents, path, configuration))
}

pub(crate) fn process_from_contents(
    contents: String,
    path: &Path,
    configuration: &Configuration,
) -> ProcessedFile {
    let ruby_contents = convert_erb_to_ruby_without_sourcemaps(contents);
    let processed_file =
        process_from_ruby_contents(ruby_contents, path, configuration);
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
