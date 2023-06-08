mod configuration;
use serde::Deserialize;
use serde::Serialize;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::PathBuf;
mod cache;
mod checker;
pub(crate) mod cli;
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct SourceLocation {
    line: usize,
    column: usize,
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Deserialize)] // Implement PartialEq trait
pub struct Pack {
    yml: PathBuf,
    name: String,
    relative_path: PathBuf,
}

impl Hash for Pack {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Implement the hash function for your struct fields
        // Call the appropriate `hash` method on the `Hasher` to hash each field
        self.name.hash(state);
    }
}
