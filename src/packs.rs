// Currently there are no supported library APIs for packs. The public API is the CLI.
// This may change in the future! Please file an issue if you have a use case for a library API.
pub mod cli;

// Module declarations
pub(crate) mod caching;
pub(crate) mod checker;
pub(crate) mod configuration;
pub(crate) mod constant_resolver;
pub(crate) mod file_utils;
pub(crate) mod logger;
pub(crate) mod pack;
pub(crate) mod pack_set;
pub(crate) mod package_todo;
pub(crate) mod parsing;
pub(crate) mod raw_configuration;
pub(crate) mod raw_pack;
pub(crate) mod reference_extractor;
pub(crate) mod walk_directory;

// Internal imports
pub(crate) use self::checker::Violation;
pub(crate) use self::pack_set::PackSet;
pub(crate) use self::parsing::process_files_with_cache;
pub(crate) use self::parsing::ruby::experimental::get_experimental_constant_resolver;
pub(crate) use self::parsing::ruby::zeitwerk::get_zeitwerk_constant_resolver;
pub(crate) use self::parsing::ParsedDefinition;
pub(crate) use self::parsing::UnresolvedReference;
pub(crate) use configuration::Configuration;
pub(crate) use package_todo::PackageTodo;

// External imports
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;

pub fn greet() {
    println!("👋 Hello! Welcome to packs 📦 🔥 🎉 🌈. This tool is under construction.")
}

pub fn list(configuration: Configuration) {
    for pack in configuration.pack_set.packs {
        println!("{}", pack.yml.display())
    }
}

pub fn delete_cache(configuration: Configuration) {
    let absolute_cache_dir = configuration.cache_directory;
    if let Err(err) = std::fs::remove_dir_all(&absolute_cache_dir) {
        eprintln!(
            "Failed to remove {}: {}",
            &absolute_cache_dir.display(),
            err
        );
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ProcessedFile {
    pub absolute_path: PathBuf,
    pub unresolved_references: Vec<UnresolvedReference>,
    pub definitions: Vec<ParsedDefinition>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Eq, Clone)]
pub struct SourceLocation {
    line: usize,
    column: usize,
}

pub(crate) fn list_definitions(configuration: &Configuration, ambiguous: bool) {
    let constant_resolver = if configuration.experimental_parser {
        let processed_files: Vec<ProcessedFile> = process_files_with_cache(
            &configuration.included_files,
            configuration.get_cache(),
            configuration,
        );

        get_experimental_constant_resolver(
            &configuration.absolute_root,
            &processed_files,
            &configuration.ignored_definitions,
        )
    } else {
        if ambiguous {
            panic!("Ambiguous mode is not supported for the Zeitwerk parser");
        }
        get_zeitwerk_constant_resolver(
            &configuration.pack_set,
            &configuration.absolute_root,
            &configuration.cache_directory,
            !configuration.cache_enabled,
        )
    };

    let constant_definition_map = constant_resolver
        .fully_qualified_constant_name_to_constant_definition_map();

    for (name, definitions) in constant_definition_map {
        if ambiguous && definitions.len() == 1 {
            continue;
        }

        for definition in definitions {
            let relative_path = definition
                .absolute_path_of_definition
                .strip_prefix(&configuration.absolute_root)
                .unwrap();

            println!("{:?} is defined at {:?}", name, relative_path);
        }
    }
}

#[cfg(test)]
mod tests {
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
        let absolute_file_path = configuration
            .absolute_root
            .join("packs/foo/app/services/foo.rb")
            .canonicalize()
            .expect("Could not canonicalize path");

        assert_eq!(
            String::from("packs/foo"),
            configuration
                .pack_set
                .for_file(&absolute_file_path)
                .unwrap()
                .name
        )
    }
}
