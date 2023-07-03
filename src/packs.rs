pub(crate) mod configuration;
mod raw_configuration;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::fs::File;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
pub(crate) mod checker;
pub mod cli;
pub(crate) mod file_utils;
mod inflector_shim;
pub mod logger;
pub(crate) mod noop_cache;
mod pack_set;
pub mod package_todo;
pub mod parsing;
pub(crate) mod per_file_cache;
mod walk_directory;

// Re-exports: Eventually, these may be part of the public API for packs
pub use crate::packs::checker::Violation;
pub use crate::packs::pack_set::PackSet;
use crate::packs::parsing::process_files_with_cache;
use crate::packs::parsing::ruby::experimental::get_experimental_constant_resolver;
use crate::packs::parsing::ruby::zeitwerk_utils::get_zeitwerk_constant_resolver;
use crate::packs::per_file_cache::create_cache_dir_idempotently;
pub use configuration::Configuration;
pub use package_todo::PackageTodo;

use self::checker::ViolationIdentifier;

use self::parsing::Definition;
use self::parsing::UnresolvedReference;

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
    pub definitions: Vec<Definition>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Eq)]
pub struct SourceLocation {
    line: usize,
    column: usize,
}

#[derive(Debug, Deserialize)]
pub struct RawPack {
    #[serde(default)]
    dependencies: HashSet<String>,
    #[serde(default)]
    ignored_dependencies: HashSet<String>,
    #[serde(default)]
    visible_to: HashSet<String>,
    #[serde(default)]
    ignored_private_constants: HashSet<String>,
    #[serde(default = "default_public_folder")]
    public_folder: String,
    #[serde(default)]
    layer: Option<String>,
    #[serde(default = "default_checker_setting")]
    enforce_dependencies: String,
    #[serde(default = "default_checker_setting")]
    enforce_privacy: String,
    #[serde(default = "default_checker_setting")]
    enforce_visibility: String,
    #[serde(default = "default_checker_setting")]
    enforce_architecture: String,
}

fn default_checker_setting() -> String {
    "false".to_string()
}

fn default_public_folder() -> String {
    "app/public".to_string()
}

// Make an enum for the configuration of a checker, which can be either false, true, or strict:
#[derive(Debug, Default, PartialEq, Eq, Deserialize, Clone)]
enum CheckerSetting {
    #[default]
    False,
    True,
    Strict,
}

impl CheckerSetting {
    /// Returns `true` if the checker setting is [`False`].
    ///
    /// [`False`]: CheckerSetting::False
    #[must_use]
    fn is_false(&self) -> bool {
        matches!(self, Self::False)
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, Clone, Default)]
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
    ignored_dependencies: HashSet<String>,
    ignored_private_constants: HashSet<String>,
    package_todo: PackageTodo,
    visible_to: HashSet<String>,
    public_folder: PathBuf,
    layer: Option<String>,
    enforce_dependencies: CheckerSetting,
    enforce_privacy: CheckerSetting,
    enforce_visibility: CheckerSetting,
    enforce_architecture: CheckerSetting,
}

impl Hash for Pack {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Implement the hash function for your struct fields
        // Call the appropriate `hash` method on the `Hasher` to hash each field
        self.name.hash(state);
    }
}

impl Pack {
    fn relative_yml(&self) -> PathBuf {
        self.relative_path.join("package.yml")
    }
}

fn convert_raw_checker_setting(raw_checker_setting: &str) -> CheckerSetting {
    match raw_checker_setting {
        "false" => CheckerSetting::False,
        "true" => CheckerSetting::True,
        "strict" => CheckerSetting::Strict,
        _ => panic!("Invalid checker setting: {}", raw_checker_setting),
    }
}

impl Pack {
    pub fn all_violations(&self) -> Vec<ViolationIdentifier> {
        let mut violations = Vec::new();
        let violations_by_pack = &self.package_todo.violations_by_defining_pack;
        for (defining_pack_name, violation_groups) in violations_by_pack {
            for (constant_name, violation_group) in violation_groups {
                for violation_type in &violation_group.violation_types {
                    for file in &violation_group.files {
                        let identifier = ViolationIdentifier {
                            violation_type: violation_type.clone(),
                            file: file.clone(),
                            constant_name: constant_name.clone(),
                            referencing_pack_name: self.name.clone(),
                            defining_pack_name: defining_pack_name.clone(),
                        };

                        violations.push(identifier);
                    }
                }
            }
        }
        violations
    }

    pub fn from_path(
        package_yml_absolute_path: &Path,
        package_yml_relative_path: &Path,
    ) -> Pack {
        let mut relative_path = package_yml_relative_path
            .parent()
            .expect("Expected package to be in a parent directory")
            .to_owned();

        let mut name = relative_path
            .to_str()
            .expect("Non-unicode characters?")
            .to_owned();
        let yml = package_yml_absolute_path;

        // Handle the root pack
        if name == *"" {
            name = String::from(".");
            relative_path = PathBuf::from(".");
        };

        let mut yaml_contents = String::new();
        let mut file = File::open(yml).expect("Failed to open the YAML file");
        file.read_to_string(&mut yaml_contents)
            .expect("Failed to read the YAML file");

        let raw_pack: RawPack = serde_yaml::from_str(&yaml_contents)
            .expect("Failed to deserialize the YAML");

        let absolute_path_to_package_todo = package_yml_absolute_path
            .parent()
            .unwrap()
            .join("package_todo.yml");

        let package_todo: PackageTodo =
            if absolute_path_to_package_todo.exists() {
                let mut package_todo_contents = String::new();
                let mut file = File::open(&absolute_path_to_package_todo)
                    .expect("Failed to open the package_todo.yml file");
                file.read_to_string(&mut package_todo_contents)
                    .expect("Could not read the package_todo.yml file");
                serde_yaml::from_str(&package_todo_contents).unwrap()
            } else {
                PackageTodo::default()
            };

        let dependencies = raw_pack.dependencies;
        let visible_to = raw_pack.visible_to;
        let public_folder = relative_path.join(raw_pack.public_folder);
        let ignored_dependencies = raw_pack.ignored_dependencies;
        let ignored_private_constants = raw_pack.ignored_private_constants;

        let enforce_dependencies =
            convert_raw_checker_setting(&raw_pack.enforce_dependencies);
        let enforce_privacy =
            convert_raw_checker_setting(&raw_pack.enforce_privacy);
        let enforce_visibility =
            convert_raw_checker_setting(&raw_pack.enforce_visibility);
        let enforce_architecture =
            convert_raw_checker_setting(&raw_pack.enforce_architecture);

        let layer = raw_pack.layer;

        let pack: Pack = Pack {
            yml: yml.to_path_buf(),
            name,
            relative_path,
            dependencies,
            package_todo,
            ignored_dependencies,
            ignored_private_constants,
            visible_to,
            enforce_dependencies,
            enforce_privacy,
            enforce_visibility,
            enforce_architecture,
            public_folder,
            layer,
        };

        pack
    }
}

pub(crate) fn list_definitions(configuration: &Configuration) {
    // TODO: Write a test that if this isn't here, it fails gracefully
    create_cache_dir_idempotently(&configuration.cache_directory);

    let constant_resolver = if configuration.experimental_parser {
        let processed_files: Vec<ProcessedFile> = process_files_with_cache(
            &configuration.absolute_root,
            &configuration.included_files,
            configuration.get_cache(),
            true,
        );

        get_experimental_constant_resolver(
            &configuration.absolute_root,
            &processed_files,
        )
    } else {
        get_zeitwerk_constant_resolver(
            &configuration.pack_set,
            &configuration.absolute_root,
            &configuration.cache_directory,
            !configuration.cache_enabled,
        )
    };

    let constants = constant_resolver
        .fully_qualified_constant_to_constant_map
        .values();

    for constant in constants {
        let relative_path = constant
            .absolute_path_of_definition
            .strip_prefix(&configuration.absolute_root)
            .unwrap();
        println!(
            "{:?} is defined at {:?}",
            constant.fully_qualified_name, relative_path
        );
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
