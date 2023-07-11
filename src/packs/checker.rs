use crate::packs::caching::create_cache_dir_idempotently;
use crate::packs::package_todo;
use crate::packs::parsing::process_files_with_cache;
use crate::packs::parsing::ruby::experimental::get_experimental_constant_resolver;
use crate::packs::parsing::ruby::zeitwerk::get_zeitwerk_constant_resolver;

use crate::packs::Configuration;
use crate::packs::ProcessedFile;

use rayon::prelude::IntoParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;

use std::collections::HashMap;
use std::{collections::HashSet, path::PathBuf};
use tracing::debug;

pub mod architecture;
pub mod dependency;
pub mod privacy;
pub mod visibility;

pub(crate) mod reference;
use reference::Reference;

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

pub(crate) trait CheckerInterface {
    fn check(
        &self,
        reference: &Reference,
        configuration: &Configuration,
    ) -> Option<Violation>;
}

pub(crate) trait ValidatorInterface {
    fn validate(&self, configuration: &Configuration) -> Option<String>;
}

// TODO: Break this function up into smaller functions
pub(crate) fn check_all(
    configuration: Configuration,
    files: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Intersecting input files with configuration included files");
    let absolute_paths: HashSet<PathBuf> = configuration.intersect_files(files);

    let found_violations: HashSet<Violation> =
        get_all_violations(&configuration, &absolute_paths);

    let recorded_violations = &configuration.pack_set.all_violations;

    debug!("Filtering out recorded violations");
    let unrecorded_violations = found_violations
        .iter()
        .filter(|v| !recorded_violations.contains(&v.identifier))
        .collect::<Vec<&Violation>>();

    debug!("Finished filtering out recorded violations");

    debug!("Finding stale violations");
    let found_violation_identifiers: HashSet<&ViolationIdentifier> =
        found_violations.par_iter().map(|v| &v.identifier).collect();

    let stale_violations = recorded_violations
        .par_iter()
        .filter(|v_identifier| {
            !found_violation_identifiers.contains(v_identifier)
        })
        .collect::<Vec<&ViolationIdentifier>>();

    debug!("Finished finding stale violations");

    let mut errors_present = false;

    if !unrecorded_violations.is_empty() {
        for violation in unrecorded_violations.iter() {
            println!("{}\n", violation.message);
        }

        println!("{} violation(s) detected:", unrecorded_violations.len());

        errors_present = true;
    }

    if !stale_violations.is_empty() {
        println!(
            "There were stale violations found, please run `packs update`"
        );
        errors_present = true;
    }

    if errors_present {
        Err("Packwerk check failed".into())
    } else {
        println!("No violations detected!");
        Ok(())
    }
}

fn validate(configuration: &Configuration) -> Vec<String> {
    debug!("Running validators against packages");
    let validators: Vec<Box<dyn ValidatorInterface + Send + Sync>> =
        vec![Box::new(dependency::Checker {})];

    let validation_errors = validators
        .iter()
        .filter_map(|v| v.validate(configuration))
        .collect();
    debug!("Finished validators against packages");

    validation_errors
}

pub(crate) fn validate_all(
    configuration: &Configuration,
) -> Result<(), Box<dyn std::error::Error>> {
    let validation_errors = validate(configuration);
    if !validation_errors.is_empty() {
        println!("{} validation error(s) detected:", validation_errors.len());
        for validation_error in validation_errors.iter() {
            println!("{}\n", validation_error);
        }

        Err("Packwerk validate failed".into())
    } else {
        println!("Packwerk validate succeeded!");
        Ok(())
    }
}

pub(crate) fn update(
    configuration: Configuration,
) -> Result<(), Box<dyn std::error::Error>> {
    let violations =
        get_all_violations(&configuration, &configuration.included_files);

    package_todo::write_violations_to_disk(configuration, violations);
    println!("Successfully updated package_todo.yml files!");
    Ok(())
}

pub(crate) fn list_unnecessary_dependencies(
    configuration: &Configuration,
) -> Result<(), Box<dyn std::error::Error>> {
    let references =
        get_all_references(configuration, &configuration.included_files);
    let mut edge_counts: HashMap<(String, String), i32> = HashMap::new();
    for reference in references {
        let defining_pack_name = reference.defining_pack_name;
        if let Some(defining_pack_name) = defining_pack_name {
            let edge_key =
                (reference.referencing_pack_name, defining_pack_name);

            edge_counts
                .entry(edge_key)
                .and_modify(|f| *f += 1)
                .or_insert(1);
        }
    }

    let mut error = false;
    for pack in &configuration.pack_set.packs {
        for dependency_name in &pack.dependencies {
            let edge_key = (pack.name.clone(), dependency_name.clone());
            let edge_count = edge_counts.get(&edge_key).unwrap_or(&0);
            if edge_count == &0 {
                error = true;
                println!(
                    "{} depends on {} but does not use it",
                    pack.name, dependency_name
                )
            }
        }
    }

    if error {
        Err("List unnecessary dependencies failed".into())
    } else {
        Ok(())
    }
}

fn get_all_violations(
    configuration: &Configuration,
    absolute_paths: &HashSet<PathBuf>,
) -> HashSet<Violation> {
    let references = get_all_references(configuration, absolute_paths);

    debug!("Running checkers on resolved references");
    let checkers: Vec<Box<dyn CheckerInterface + Send + Sync>> = vec![
        Box::new(dependency::Checker {}),
        Box::new(privacy::Checker {}),
        Box::new(visibility::Checker {}),
        Box::new(architecture::Checker {
            layers: configuration.layers.clone(),
        }),
    ];

    let violations: HashSet<Violation> = checkers
        .into_par_iter()
        .flat_map(|c| {
            references
                .par_iter()
                .flat_map(|r| c.check(r, configuration))
                .collect::<HashSet<Violation>>()
        })
        .collect();

    debug!("Finished running checkers");

    violations
}

fn get_all_references(
    configuration: &Configuration,
    absolute_paths: &HashSet<PathBuf>,
) -> Vec<Reference> {
    let initialized_dir =
        create_cache_dir_idempotently(&configuration.cache_directory);

    let cache = configuration.get_cache(initialized_dir);

    debug!("Getting unresolved references (using cache if possible)");
    let processed_files: Vec<ProcessedFile> = process_files_with_cache(
        &configuration.absolute_root,
        absolute_paths,
        cache,
        configuration,
    );

    let constant_resolver = if configuration.experimental_parser {
        get_experimental_constant_resolver(
            &configuration.absolute_root,
            &processed_files,
            &configuration.ignored_definitions,
        )
    } else {
        get_zeitwerk_constant_resolver(
            &configuration.pack_set,
            &configuration.absolute_root,
            &configuration.cache_directory,
            !configuration.cache_enabled,
        )
    };

    debug!("Turning unresolved references into fully qualified references");
    let references: Vec<Reference> = processed_files
        .par_iter()
        .flat_map(|processed_file| {
            let references: Vec<Reference> = processed_file
                .unresolved_references
                .iter()
                .flat_map(|unresolved_ref| {
                    let absolute_path_of_referring_file =
                        processed_file.absolute_path.clone();
                    Reference::from_unresolved_reference(
                        configuration,
                        constant_resolver.as_ref(),
                        unresolved_ref,
                        &absolute_path_of_referring_file,
                    )
                })
                .collect::<Vec<Reference>>();

            references
        })
        .collect();

    debug!("Finished turning unresolved references into fully qualified references");

    references
}
