// Currently there are no supported library APIs for packs. The public API is the CLI.
// This may change in the future! Please file an issue if you have a use case for a library API.
pub mod cli;

// Module declarations
pub(crate) mod caching;
pub(crate) mod checker;
pub(crate) mod configuration;
pub(crate) mod constant_resolver;
pub(crate) mod dependencies;
pub(crate) mod ignored;
pub(crate) mod monkey_patch_detection;
pub(crate) mod pack;
pub(crate) mod parsing;
pub(crate) mod raw_configuration;
pub(crate) mod walk_directory;

mod constant_dependencies;
mod file_utils;
mod logger;
mod pack_set;
mod package_todo;
mod reference_extractor;

use crate::packs;
use crate::packs::pack::write_pack_to_disk;
use crate::packs::pack::Pack;

// Internal imports
pub(crate) use self::checker::Violation;
pub(crate) use self::pack_set::PackSet;
pub(crate) use self::parsing::process_files_with_cache;
pub(crate) use self::parsing::ruby::experimental::get_experimental_constant_resolver;
pub(crate) use self::parsing::ruby::zeitwerk::get_zeitwerk_constant_resolver;
pub(crate) use self::parsing::ParsedDefinition;
pub(crate) use self::parsing::UnresolvedReference;
use anyhow::bail;
pub(crate) use configuration::Configuration;
pub(crate) use package_todo::PackageTodo;

// External imports
use anyhow::Context;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;

pub fn greet() {
    println!("👋 Hello! Welcome to packs 📦 🔥 🎉 🌈. This tool is under construction.")
}

fn create(configuration: &Configuration, name: String) -> anyhow::Result<()> {
    let existing_pack = configuration.pack_set.for_pack(&name);
    if existing_pack.is_ok() {
        println!("`{}` already exists!", &name);
        return Ok(());
    }
    let new_pack_path =
        configuration.absolute_root.join(&name).join("package.yml");

    let new_pack = Pack::from_contents(
        &new_pack_path,
        &configuration.absolute_root,
        "enforce_dependencies: true",
        PackageTodo::default(),
    )?;

    write_pack_to_disk(&new_pack)?;

    let readme = format!(
"Welcome to `{}`!

If you're the author, please consider replacing this file with a README.md, which may contain:
- What your pack is and does
- How you expect people to use your pack
- Example usage of your pack's public API and where to find it
- Limitations, risks, and important considerations of usage
- How to get in touch with eng and other stakeholders for questions or issues pertaining to this pack
- What SLAs/SLOs (service level agreements/objectives), if any, your package provides
- When in doubt, keep it simple
- Anything else you may want to include!

README.md should change as your public API changes.

See https://github.com/rubyatscale/packs#readme for more info!",
    new_pack.name
);

    let readme_path = configuration.absolute_root.join(&name).join("README.md");
    std::fs::write(readme_path, readme).context("Failed to write README.md")?;

    println!("Successfully created `{}`!", name);
    Ok(())
}

pub fn check(
    configuration: &Configuration,
    files: Vec<String>,
) -> anyhow::Result<()> {
    let result = checker::check_all(configuration, files)
        .context("Failed to check files")?;
    println!("{}", result);
    if result.has_violations() {
        bail!("Violations found!")
    }
    Ok(())
}

pub fn update(configuration: &Configuration) -> anyhow::Result<()> {
    checker::update(configuration)
}

pub fn add_dependency(
    configuration: &Configuration,
    from: String,
    to: String,
) -> anyhow::Result<()> {
    let pack_set = &configuration.pack_set;

    let from_pack = pack_set
        .for_pack(&from)
        .context(format!("`{}` not found", from))?;

    let to_pack = pack_set
        .for_pack(&to)
        .context(format!("`{}` not found", to))?;

    // Print a warning if the dependency already exists
    if from_pack.dependencies.contains(&to_pack.name) {
        println!(
            "`{}` already depends on `{}`!",
            from_pack.name, to_pack.name
        );
        return Ok(());
    }

    let new_from_pack = from_pack.add_dependency(to_pack);

    write_pack_to_disk(&new_from_pack)?;

    // Note: Ideally we wouldn't have to refetch the configuration and could instead
    // either update the existing one OR modify the existing one and return a new one
    // (which takes ownership over the previous one).
    // For now, we simply refetch the entire configuration for simplicity,
    // since we don't mind the slowdown for this CLI command.
    let new_configuration = configuration::get(&configuration.absolute_root)?;
    let validation_result = packs::validate(&new_configuration);
    if validation_result.is_err() {
        println!("Added `{}` as a dependency to `{}`!", to, from);
        println!("Warning: This creates a cycle!");
    } else {
        println!("Successfully added `{}` as a dependency to `{}`!", to, from);
    }

    Ok(())
}

pub fn list_included_files(configuration: Configuration) -> anyhow::Result<()> {
    configuration
        .included_files
        .iter()
        .for_each(|f| println!("{}", f.display()));
    Ok(())
}

pub fn validate(configuration: &Configuration) -> anyhow::Result<()> {
    checker::validate_all(configuration)
}

pub fn configuration(project_root: PathBuf) -> anyhow::Result<Configuration> {
    let absolute_root = project_root.canonicalize()?;
    configuration::get(&absolute_root)
}

pub fn check_unnecessary_dependencies(
    configuration: &Configuration,
    auto_correct: bool,
) -> anyhow::Result<()> {
    if auto_correct {
        checker::remove_unnecessary_dependencies(configuration)
    } else {
        checker::check_unnecessary_dependencies(configuration)
    }
}

pub fn update_dependencies_for_constant(
    configuration: &Configuration,
    constant_name: &str,
) -> anyhow::Result<()> {
    match constant_dependencies::update_dependencies_for_constant(
        configuration,
        constant_name,
    ) {
        Ok(num_updated) => {
            match num_updated {
                0 => println!(
                    "No dependencies to update for constant '{}'",
                    constant_name
                ),
                1 => println!(
                    "Successfully updated 1 dependency for constant '{}'",
                    constant_name
                ),
                _ => println!(
                    "Successfully updated {} dependencies for constant '{}'",
                    num_updated, constant_name
                ),
            }
            Ok(())
        }
        Err(err) => Err(anyhow::anyhow!(err)),
    }
}

pub fn list(configuration: Configuration) {
    for pack in configuration.pack_set.packs {
        println!("{}", pack.yml.display())
    }
}

pub fn lint_package_yml_files(
    configuration: &Configuration,
) -> anyhow::Result<()> {
    for pack in &configuration.pack_set.packs {
        write_pack_to_disk(pack)?
    }
    Ok(())
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

pub(crate) fn list_definitions(
    configuration: &Configuration,
    ambiguous: bool,
) -> anyhow::Result<()> {
    let constant_resolver = if configuration.experimental_parser {
        let processed_files: Vec<ProcessedFile> = process_files_with_cache(
            &configuration.included_files,
            configuration.get_cache(),
            configuration,
        )?;

        get_experimental_constant_resolver(
            &configuration.absolute_root,
            &processed_files,
            &configuration.ignored_definitions,
        )
    } else {
        if ambiguous {
            bail!("Ambiguous mode is not supported for the Zeitwerk parser");
        }
        get_zeitwerk_constant_resolver(
            &configuration.pack_set,
            &configuration.constant_resolver_configuration(),
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
                .strip_prefix(&configuration.absolute_root)?;

            println!("{:?} is defined at {:?}", name, relative_path);
        }
    }
    Ok(())
}

fn expose_monkey_patches(
    configuration: &Configuration,
    rubydir: &PathBuf,
    gemdir: &PathBuf,
) -> anyhow::Result<()> {
    println!(
        "{}",
        monkey_patch_detection::expose_monkey_patches(
            configuration,
            rubydir,
            gemdir,
        )?
    );
    Ok(())
}

fn list_dependencies(
    configuration: &Configuration,
    pack_name: String,
) -> anyhow::Result<()> {
    println!("Pack dependencies for {}\n", pack_name);
    let dependencies =
        dependencies::find_dependencies(configuration, &pack_name)?;
    println!("Explicit ({}):", dependencies.explicit.len());
    if dependencies.explicit.is_empty() {
        println!("- None");
    } else {
        for dependency in dependencies.explicit {
            println!("- {}", dependency);
        }
    }
    println!("\nImplicit (violations) ({}):", dependencies.implicit.len());
    if dependencies.implicit.is_empty() {
        println!("- None");
    } else {
        let mut dependent_packs_with_violations =
            dependencies.implicit.keys().collect::<Vec<_>>();
        dependent_packs_with_violations.sort();
        for dependent in dependent_packs_with_violations {
            println!("- {}", dependent);
            for (violation_type, count) in &dependencies.implicit[dependent] {
                println!("  - {}: {}", violation_type, count);
            }
        }
    }
    Ok(())
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
        )
        .unwrap();
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
                .unwrap()
                .name
        )
    }
}
