// Currently there are no supported library APIs for packs. The public API is the CLI.
// This may change in the future! Please file an issue if you have a use case for a library API.
pub mod cli;

// Module declarations
pub(crate) mod bin_locater;
pub(crate) mod caching;
pub(crate) mod checker;
pub(crate) mod configuration;
pub(crate) mod constant_resolver;
pub(crate) mod dependencies;
pub(crate) mod ignored;
pub(crate) mod monkey_patch_detection;
pub mod pack;
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
use std::path::{Path, PathBuf};

pub fn greet() {
    println!("👋 Hello! Welcome to packs 📦 🔥 🎉 🌈. This tool is under construction.")
}

pub fn init(absolute_root: &Path, use_packwerk: bool) -> anyhow::Result<()> {
    let command = if use_packwerk { "packwerk" } else { "pks" };
    let root_package = format!("\
# This file represents the root package of the application
# Please validate the configuration using `{} validate` (for Rails applications) or running the auto generated
# test case (for non-Rails projects). You can then use `{} check` to check your code.

# Change to `true` to turn on dependency checks for this package
enforce_dependencies: false

# A list of this package's dependencies
# Note that packages in this list require their own `package.yml` file
# dependencies:
# - \"packages/billing\"
", command, command);

    let packs_config = "\
# See: Setting up the configuration file
# https://github.com/Shopify/packwerk/blob/main/USAGE.md#configuring-packwerk

# List of patterns for folder paths to include
# include:
# - \"**/*.{rb,rake,erb}\"

# List of patterns for folder paths to exclude
# exclude:
# - \"{bin,node_modules,script,tmp,vendor}/**/*\"

# Patterns to find package configuration files
# package_paths: \"**/\"

# List of custom associations, if any
# custom_associations:
# - \"cache_belongs_to\"

# Whether or not you want the cache enabled (disabled by default)
# cache: true

# Where you want the cache to be stored (default below)
# cache_directory: \"tmp/cache/packwerk\"
";
    let root_package_path = absolute_root.join("package.yml");
    let packs_config_path = absolute_root.join(if use_packwerk {
        "packwerk.yml"
    } else {
        "packs.yml"
    });

    if root_package_path.exists() {
        println!("`{}` already exists!", root_package_path.display());
        bail!("Could not initialize package.yml")
    }
    if packs_config_path.exists() {
        println!("`{}` already exists!", packs_config_path.display());
        bail!(format!(
            "Could not initialize {}",
            packs_config_path.display()
        ))
    }

    std::fs::write(root_package_path.clone(), root_package).unwrap();
    std::fs::write(packs_config_path.clone(), packs_config).unwrap();

    println!(
        "Created '{}' and '{}'",
        packs_config_path.display(),
        root_package_path.display()
    );
    Ok(())
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
    json: bool,
) -> anyhow::Result<()> {
    let result = checker::check_all(configuration, files)
        .context("Failed to check files")?;
    if json {
        println!("{}", result.to_json().context("Failed to serialize JSON")?);
        if result.has_violations() {
            std::process::exit(1);
        }
    } else {
        println!("{}", result);
        if result.has_violations() {
            let count = result.violation_count();
            bail!("{} violation(s) found!", count)
        }
    }
    Ok(())
}

pub fn update(
    configuration: &Configuration,
    options: &checker::UpdateOptions,
) -> anyhow::Result<()> {
    // Debug log configuration if ENV variable PACKS_DEBUG is set
    if std::env::var("PACKS_DEBUG").is_ok() {
        println!("Configuration: {:#?}", configuration);
    }
    checker::update(configuration, options)
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
    let new_configuration = configuration::get(
        &configuration.absolute_root,
        &configuration.input_files_count,
    )?;
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

pub fn configuration(
    project_root: PathBuf,
    input_files_count: &usize,
) -> anyhow::Result<Configuration> {
    let absolute_root = project_root.canonicalize()?;
    configuration::get(&absolute_root, input_files_count)
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

pub fn add_dependencies(
    configuration: &Configuration,
    pack_name: &str,
) -> anyhow::Result<()> {
    checker::add_all_dependencies(configuration, pack_name)
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

pub fn lint(configuration: &Configuration) -> anyhow::Result<()> {
    // Lint package.yml files
    for pack in &configuration.pack_set.packs {
        write_pack_to_disk(pack)?
    }
    // Lint package_todo.yml files
    package_todo::lint_package_todo_yml_files(configuration);
    Ok(())
}

#[deprecated(note = "Use lint() instead")]
pub fn lint_package_yml_files(
    configuration: &Configuration,
) -> anyhow::Result<()> {
    lint(configuration)
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

    #[serde(default)] // Default to an empty Vec if not present
    pub sigils: Vec<Sigil>,
}

// A sigil is a way to specify some packs specific behavior at the top of a file, like
// `# pack_public: true`. This struct picks up sigil names, which are an enum of string values, starting with just one possibility.
// and value, which is a boolean
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Sigil {
    pub name: String,
    pub value: bool,
}

#[derive(
    Debug, PartialEq, Serialize, Deserialize, Default, Eq, Clone, Hash,
)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
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

pub(crate) fn list_references(
    configuration: &Configuration,
    format: &str,
    output_file: Option<&Path>,
) -> anyhow::Result<()> {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;

    // Get all processed files
    let processed_files: Vec<ProcessedFile> = process_files_with_cache(
        &configuration.included_files,
        configuration.get_cache(),
        configuration,
    )?;

    // Get constant resolver
    let constant_resolver = if configuration.experimental_parser {
        get_experimental_constant_resolver(
            &configuration.absolute_root,
            &processed_files,
            &configuration.ignored_definitions,
        )
    } else {
        get_zeitwerk_constant_resolver(
            &configuration.pack_set,
            &configuration.constant_resolver_configuration(),
        )
    };

    // Build map: source_file -> { constant_name -> definition_file }
    let mut reference_map: HashMap<String, HashMap<String, String>> =
        HashMap::new();

    for processed_file in &processed_files {
        let relative_source_path = processed_file
            .absolute_path
            .strip_prefix(&configuration.absolute_root)?
            .to_string_lossy()
            .to_string();

        let mut constants_in_file: HashMap<String, String> = HashMap::new();

        // Get all unresolved references from this file and resolve them
        for unresolved_ref in &processed_file.unresolved_references {
            // Resolve the reference to a fully qualified constant and its definition
            let references =
                checker::reference::Reference::from_unresolved_reference(
                    configuration,
                    constant_resolver.as_ref(),
                    unresolved_ref,
                    &processed_file.absolute_path,
                )?;

            // Each unresolved reference might resolve to multiple definitions
            for reference in references {
                let constant_name = reference.constant_name.clone();

                // Only include references that have a defining file
                if let Some(relative_def_path) =
                    reference.relative_defining_file
                {
                    constants_in_file.insert(constant_name, relative_def_path);
                }
            }
        }

        if !constants_in_file.is_empty() {
            reference_map.insert(relative_source_path, constants_in_file);
        }
    }

    // Output the results
    let output = match format {
        "json" => serde_json::to_string_pretty(&reference_map)?,
        "text" => {
            let mut lines = Vec::new();
            for (source_file, constants) in &reference_map {
                lines.push(format!("{}:", source_file));
                for (const_name, def_file) in constants {
                    lines.push(format!("  {} => {}", const_name, def_file));
                }
            }
            lines.join("\n")
        }
        _ => bail!("Unsupported format: {}. Use 'json' or 'text'", format),
    };

    // Write to file or stdout
    if let Some(path) = output_file {
        let mut file = File::create(path)?;
        file.write_all(output.as_bytes())?;
        println!("Reference map written to: {}", path.display());
    } else {
        println!("{}", output);
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

fn move_to_pack(
    configuration: &Configuration,
    destination: &str,
    paths: Vec<String>,
) -> anyhow::Result<()> {
    let dest_pack = configuration
        .pack_set
        .for_pack(destination)
        .context(format!("Cannot move to '{}': pack not found", destination))?;
    let dest_relative_path = dest_pack.relative_path.clone();

    // Expand input paths: if a path is a directory, glob all files within it
    let mut source_files: Vec<PathBuf> = Vec::new();
    for path_str in &paths {
        let path_str = path_str.trim_end_matches('/');
        let absolute_path = configuration.absolute_root.join(path_str);
        if absolute_path.is_dir() {
            let pattern = absolute_path.join("**/*.*");
            let entries = glob::glob(pattern.to_str().unwrap())
                .context("Failed to glob")?;
            for entry in entries.flatten() {
                let relative = entry
                    .strip_prefix(&configuration.absolute_root)
                    .unwrap()
                    .to_path_buf();
                let filename =
                    relative.file_name().unwrap().to_string_lossy().to_string();
                if filename != "package.yml" && filename != "package_todo.yml" {
                    source_files.push(relative);
                }
            }
        } else {
            source_files.push(PathBuf::from(path_str));
        }
    }

    // Compute file move operations
    struct FileMoveOperation {
        origin: PathBuf,
        destination: PathBuf,
    }

    let mut operations: Vec<FileMoveOperation> = Vec::new();

    for source_file in &source_files {
        let source_str = source_file.to_string_lossy().to_string();

        // Find the origin pack (longest prefix match).
        // pack_set.packs is already sorted by name length descending.
        let origin_pack = configuration.pack_set.packs.iter().find(|p| {
            p.name != "."
                && (source_str
                    .starts_with(&format!("{}/", p.relative_path.display()))
                    || source_str == p.relative_path.to_string_lossy())
        });

        let dest_path = if let Some(origin) = origin_pack {
            let origin_prefix = format!("{}/", origin.relative_path.display());
            if let Some(remainder) = source_str.strip_prefix(&origin_prefix) {
                dest_relative_path.join(remainder)
            } else {
                dest_relative_path.join(&source_str)
            }
        } else {
            dest_relative_path.join(&source_str)
        };

        // Compute origin pack name for reference updating later
        operations.push(FileMoveOperation {
            origin: source_file.clone(),
            destination: dest_path.clone(),
        });

        // Auto-detect corresponding spec file
        let within_pack = if let Some(origin) = origin_pack {
            let origin_prefix = format!("{}/", origin.relative_path.display());
            source_str
                .strip_prefix(&origin_prefix)
                .unwrap_or(&source_str)
                .to_string()
        } else {
            source_str.clone()
        };

        let spec_origin_within_pack = compute_spec_path(&within_pack);

        if let Some(spec_relative) = spec_origin_within_pack {
            let spec_origin = if let Some(origin) = origin_pack {
                origin.relative_path.join(&spec_relative)
            } else {
                PathBuf::from(&spec_relative)
            };
            let spec_dest = dest_relative_path.join(&spec_relative);

            operations.push(FileMoveOperation {
                origin: spec_origin,
                destination: spec_dest,
            });
        }
    }

    // Step 4: Move files
    println!("{}", "=".repeat(100));
    println!("File Operations");

    let mut moved_pairs: Vec<(String, String)> = Vec::new();

    for op in &operations {
        let origin_abs = configuration.absolute_root.join(&op.origin);
        let dest_abs = configuration.absolute_root.join(&op.destination);
        let origin_exists = origin_abs.exists();
        let dest_exists = dest_abs.exists();

        if origin_exists && dest_exists {
            println!(
                "[SKIP] Not moving {}, {} already exists",
                op.origin.display(),
                op.destination.display()
            );
        } else if origin_exists && !dest_exists {
            if let Some(parent) = dest_abs.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::rename(&origin_abs, &dest_abs)?;
            println!(
                "Moving file {} to {}",
                op.origin.display(),
                op.destination.display()
            );
            moved_pairs.push((
                op.origin.to_string_lossy().to_string(),
                op.destination.to_string_lossy().to_string(),
            ));
        } else if !origin_exists && dest_exists {
            println!(
                "[SKIP] Not moving {}, does not exist, ({} already exists)",
                op.origin.display(),
                op.destination.display()
            );
        }
        // If neither exists: silent (no output, same as Ruby)
    }

    // Step 5: Update .rubocop_todo.yml
    let rubocop_todo_path =
        configuration.absolute_root.join(".rubocop_todo.yml");
    if rubocop_todo_path.exists() {
        let mut contents = std::fs::read_to_string(&rubocop_todo_path)?;
        for (origin, dest) in &moved_pairs {
            let count = contents.matches(origin.as_str()).count();
            if count > 0 {
                contents = contents.replace(origin.as_str(), dest.as_str());
                println!(
                    "Replaced {} occurrence(s) of {} in .rubocop_todo.yml",
                    count, origin
                );
            }
        }
        std::fs::write(&rubocop_todo_path, contents)?;
    }

    // Step 6: Update references across the repo
    // Collect unique origin_pack_name -> dest_pack_name pairs
    let mut pack_renames: Vec<(String, String)> = Vec::new();
    for source_file in &source_files {
        let source_str = source_file.to_string_lossy().to_string();
        let origin_pack = configuration.pack_set.packs.iter().find(|p| {
            p.name != "."
                && source_str
                    .starts_with(&format!("{}/", p.relative_path.display()))
        });

        if let Some(origin) = origin_pack {
            let rename = (
                origin.name.clone(),
                destination.trim_end_matches('/').to_string(),
            );
            if !pack_renames.contains(&rename) && rename.0 != rename.1 {
                pack_renames.push(rename);
            }
        }
    }

    for (origin_name, dest_name) in &pack_renames {
        update_references_in_project(
            &configuration.absolute_root,
            origin_name,
            dest_name,
        )?;
    }

    Ok(())
}

fn compute_spec_path(within_pack_path: &str) -> Option<String> {
    if within_pack_path.starts_with("app/") {
        // app/services/foo/bar.rb -> spec/services/foo/bar_spec.rb
        let without_app = within_pack_path.strip_prefix("app/")?;
        let without_ext = without_app.strip_suffix(".rb")?;
        Some(format!("spec/{}_spec.rb", without_ext))
    } else if within_pack_path.starts_with("lib/") {
        // lib/foo.rb -> spec/lib/foo_spec.rb
        let without_ext = within_pack_path.strip_suffix(".rb")?;
        Some(format!("spec/{}_spec.rb", without_ext))
    } else {
        None
    }
}

fn update_references_in_project(
    absolute_root: &Path,
    find: &str,
    replace: &str,
) -> anyhow::Result<()> {
    use jwalk::WalkDir;

    for entry in WalkDir::new(absolute_root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() {
            continue;
        }

        let path = entry.path();

        // Skip binary files and common non-text directories
        let relative = path
            .strip_prefix(absolute_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        if relative.starts_with("node_modules/")
            || relative.starts_with("vendor/")
            || relative.starts_with("tmp/")
            || relative.starts_with(".git/")
            || relative.starts_with("log/")
        {
            continue;
        }

        if let Ok(contents) = std::fs::read_to_string(&path) {
            let count = contents.matches(find).count();
            if count > 0 {
                let new_contents = contents.replace(find, replace);
                std::fs::write(&path, new_contents)?;
                println!(
                    "Replaced {} occurrence(s) of {} in {}",
                    count, find, relative
                );
            }
        }
    }

    Ok(())
}

fn for_file(configuration: &Configuration, file: String) -> anyhow::Result<()> {
    let absolute_file_path =
        file_utils::get_absolute_path(file.clone(), configuration);

    match configuration.pack_set.for_file(&absolute_file_path)? {
        Some(pack) => {
            println!("{}", pack.yml.display());
            Ok(())
        }
        None => {
            bail!("No pack found for file: {}", file)
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
            &10,
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
