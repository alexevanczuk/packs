use crate::packs::package_todo;
use crate::packs::parsing::process_files_with_cache;
use crate::packs::parsing::ruby::zeitwerk_utils::get_zeitwerk_constant_resolver;
use crate::packs::per_file_cache;
use crate::packs::per_file_cache::create_cache_dir_idempotently;
use crate::packs::Configuration;
use crate::packs::ProcessedFile;
use crate::packs::SourceLocation;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::ParallelIterator;
use std::path::Path;
use std::{collections::HashSet, path::PathBuf};
use tracing::debug;

use super::parsing::{
    ruby::packwerk::constant_resolver::ConstantResolver, Cache,
};
use super::Pack;
use super::UnresolvedReference;

pub mod architecture;
pub mod dependency;
pub mod privacy;
pub mod visibility;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct ViolationIdentifier {
    pub violation_type: String,
    pub file: String,
    pub constant_name: String,
    pub referencing_pack_name: String,
    pub defining_pack_name: String,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Violation {
    message: String,
    pub identifier: ViolationIdentifier,
}

#[derive(Debug)]
pub struct Reference<'a> {
    constant_name: String,
    defining_pack: Option<&'a Pack>,
    relative_defining_file: Option<String>,
    referencing_pack: &'a Pack,
    relative_referencing_file: String,
    source_location: SourceLocation,
}
impl<'a> Reference<'a> {
    fn from_unresolved_reference(
        configuration: &'a Configuration,
        constant_resolver: &'a ConstantResolver,
        unresolved_reference: &UnresolvedReference,
        referencing_file_path: &Path,
    ) -> Reference<'a> {
        let str_references: Vec<&str> = unresolved_reference
            .namespace_path
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();

        let maybe_constant = constant_resolver
            .resolve(&unresolved_reference.name, &str_references);

        let (defining_pack, relative_defining_file) = if let Some(constant) =
            &maybe_constant
        {
            let absolute_path_of_definition =
                &constant.absolute_path_of_definition;
            let relative_defining_file = absolute_path_of_definition
                .strip_prefix(&configuration.absolute_root)
                .unwrap()
                .to_path_buf()
                .to_str()
                .unwrap()
                .to_string();

            let defining_pack_name =
                configuration.pack_set.for_file(absolute_path_of_definition);

            let defining_pack: Option<&'a Pack> = match defining_pack_name {
                Some(name) => Some(configuration.pack_set.for_pack(&name)),
                None => None,
            };

            (defining_pack, Some(relative_defining_file))
        } else {
            (None, None)
        };

        let constant_name = if let Some(constant) = &maybe_constant {
            &constant.fully_qualified_name
        } else {
            // Contant name is not known, so we'll just use the unresolved name for now
            &unresolved_reference.name
        };

        let constant_name = constant_name.clone();

        let referencing_pack_name = configuration
            .pack_set
            .for_file(referencing_file_path)
            .unwrap_or_else(|| {
                panic!(
                    "Could not find pack for referencing file path: {}",
                    &referencing_file_path.display()
                )
            });

        let referencing_pack =
            configuration.pack_set.for_pack(&referencing_pack_name);

        let loc = unresolved_reference.location.clone();
        let source_location = SourceLocation {
            line: loc.start_row,
            column: loc.start_col,
        };

        let relative_referencing_file_path = referencing_file_path
            .strip_prefix(&configuration.absolute_root)
            .unwrap()
            .to_path_buf();

        let relative_referencing_file =
            relative_referencing_file_path.to_str().unwrap().to_string();

        Reference {
            constant_name,
            defining_pack,
            referencing_pack,
            relative_referencing_file,
            source_location,
            relative_defining_file,
        }
    }
}

pub(crate) trait CheckerInterface {
    fn check(&self, reference: &Reference) -> Option<Violation>;
}

// TODO: Break this function up into smaller functions
pub(crate) fn check(
    configuration: Configuration,
    files: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let cache = per_file_cache::PerFileCache {
        cache_dir: configuration.cache_directory.to_owned(),
    };

    debug!(
        target: "perf_events",
        "Interecting input files with configuration included files"
    );
    let absolute_paths: HashSet<PathBuf> = configuration.intersect_files(files);

    let violations: Vec<Violation> =
        get_all_violations(&configuration, absolute_paths, cache);
    let recorded_violations = configuration.pack_set.all_violations;

    debug!(
        target: "perf_events",
        "Filtering out recorded violations"
    );
    let unrecorded_violations = violations
        .iter()
        .filter(|v| !recorded_violations.contains(&v.identifier))
        .collect::<Vec<&Violation>>();

    debug!(
        target: "perf_events",
        "Finished filtering out recorded violations"
    );

    if !unrecorded_violations.is_empty() {
        for violation in unrecorded_violations.iter() {
            println!("{}\n", violation.message);
        }

        println!("{} violation(s) detected:", unrecorded_violations.len());
        Err("Packwerk check failed".into())
    } else {
        println!("No violations detected!");
        Ok(())
    }
}

pub(crate) fn update(
    configuration: Configuration,
) -> Result<(), Box<dyn std::error::Error>> {
    let cache = per_file_cache::PerFileCache {
        cache_dir: configuration.cache_directory.to_owned(),
    };

    let violations = get_all_violations(
        &configuration,
        configuration.intersect_files(vec![]),
        cache,
    );

    package_todo::write_violations_to_disk(configuration, violations);
    println!("Successfully updated package_todo.yml files!");
    Ok(())
}

fn get_all_violations<T: Cache + Send + Sync>(
    configuration: &Configuration,
    absolute_paths: HashSet<PathBuf>,
    cache: T,
) -> Vec<Violation> {
    // TODO: Write a test that if this isn't here, it fails gracefully
    create_cache_dir_idempotently(&configuration.cache_directory);

    debug!(
        target: "perf_events",
        "Getting unresolved references (using cache if possible)"
    );
    let processed_files: Vec<ProcessedFile> = process_files_with_cache(
        &configuration.absolute_root,
        &absolute_paths,
        cache,
    );

    let constant_resolver = get_zeitwerk_constant_resolver(
        &configuration.pack_set,
        &configuration.absolute_root,
        &configuration.cache_directory,
    );

    debug!(
        target: "perf_events",
        "Turning unresolved references into fully qualified references"
    );
    let references: Vec<Reference> = processed_files
        .into_par_iter()
        .flat_map(|processed_file| {
            let references: Vec<Reference> = processed_file
                .unresolved_references
                .iter()
                .map(|unresolved_ref| {
                    let absolute_path_of_referring_file =
                        processed_file.absolute_path.clone();
                    Reference::from_unresolved_reference(
                        configuration,
                        &constant_resolver,
                        unresolved_ref,
                        &absolute_path_of_referring_file,
                    )
                })
                .collect::<Vec<Reference>>();

            references
        })
        .collect();
    debug!(target: "perf_events", "Finished turning unresolved references into fully qualified references");

    debug!(
        target: "perf_events",
        "Running checkers on resolved references"
    );
    let checkers: Vec<Box<dyn CheckerInterface + Send + Sync>> = vec![
        Box::new(dependency::Checker {}),
        Box::new(privacy::Checker {}),
        Box::new(visibility::Checker {}),
        Box::new(architecture::Checker {
            layers: configuration.layers.clone(),
        }),
    ];
    let violations: Vec<Violation> = references
        .into_par_iter()
        .flat_map(|r| {
            checkers
                .iter()
                .flat_map(|c| c.check(&r))
                .collect::<Vec<Violation>>()
        })
        .collect();
    debug!(target: "perf_events", "Finished running checkers");

    violations
}
