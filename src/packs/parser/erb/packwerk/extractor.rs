use tree_sitter::Parser;

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
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_embedded_template::language())
        .expect("Error loading ERB grammar");
    let parsed = parser.parse(&contents, None);

    if let Some(tree) = parsed {
        // dbg!(tree.root_node());
        // Parse the tree to get a list of Ruby constant references??
        // tree.walk();
    }
    vec![]
}
