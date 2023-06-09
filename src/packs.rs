mod configuration;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::Path;
use std::path::PathBuf;
mod cache;
mod checker;
pub(crate) mod cli;
mod inflector_shim;
pub mod parser;
mod string_helpers;

// Re-exports: Eventually, these may be part of the public API for packs
pub use crate::packs::checker::Violation;
pub use configuration::Configuration;
pub use parser::ruby::packwerk::extractor::Range;
pub use parser::ruby::packwerk::extractor::UnresolvedReference;

pub fn greet() {
    println!("👋 Hello! Welcome to packs 📦 🔥 🎉 🌈. This tool is under construction.")
}

pub fn list(configuration: Configuration) {
    for pack in configuration.packs {
        println!("{}", pack.yml.display())
    }
}

// Implement for_file, which accepts the config and absolute file path.
// We can call configuration.packs, which is sorted, to find the pack with the longest
// name that is a subpath of the input absolute file path.
// If we find a pack, return it. Otherwise, return the root pack.
// Add lifetime specifier to configuration::Configuration
pub fn for_file(
    configuration: &Configuration,
    absolute_file_path: &Path,
) -> Option<String> {
    for pack in &configuration.packs {
        if absolute_file_path.starts_with(pack.yml.parent().unwrap()) {
            return Some(pack.name.clone());
        }
    }

    None
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct SourceLocation {
    line: usize,
    column: usize,
}

#[derive(Debug, Deserialize)]
pub struct DeserializablePack {
    #[serde(default)]
    dependencies: HashSet<String>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
pub struct Pack {
    #[serde(skip_deserializing)]
    yml: PathBuf,
    #[serde(skip_deserializing)]
    name: String,
    #[serde(skip_deserializing)]
    relative_path: PathBuf,
    #[serde(default)]
    // I want to see if checkers and such can add their own deserialization
    // behavior to Pack via a trait or something? That would make extension simpler!
    dependencies: HashSet<String>,
}

impl Hash for Pack {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Implement the hash function for your struct fields
        // Call the appropriate `hash` method on the `Hasher` to hash each field
        self.name.hash(state);
    }
}

// Add a test that packs::for_file(config, absolute_file_path) returns the correct pack
#[cfg(test)]
mod tests {
    use crate::packs;

    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_for_file() {
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/simple_app")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
        );
        let absolute_file_path = PathBuf::from(
            "tests/fixtures/simple_app/packs/foo/app/services/foo.rb",
        )
        .canonicalize()
        .expect("Could not canonicalize path");

        assert_eq!(
            Some(String::from("packs/foo")),
            packs::for_file(&configuration, &absolute_file_path)
        )
    }
}
