use crate::packs::caching::create_cache_dir_idempotently;
use crate::packs::package_todo;
use crate::packs::parsing::process_files_with_cache;
use crate::packs::parsing::ruby::experimental::get_experimental_constant_resolver;
use crate::packs::parsing::ruby::zeitwerk::get_zeitwerk_constant_resolver;

use crate::packs::Configuration;
use crate::packs::ProcessedFile;
use crate::packs::SourceLocation;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use std::path::Path;
use std::{collections::HashSet, path::PathBuf};
use tracing::debug;

use super::caching::Cache;
use super::pack::Pack;
use super::parsing::ruby::zeitwerk::constant_resolver::ZeitwerkConstantResolver;
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
        constant_resolver: &'a ZeitwerkConstantResolver,
        unresolved_reference: &UnresolvedReference,
        referencing_file_path: &Path,
    ) -> Reference<'a> {
        let str_namespace_path: Vec<&str> = unresolved_reference
            .namespace_path
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();

        let maybe_constant = constant_resolver
            .resolve(&unresolved_reference.name, &str_namespace_path);

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

            let defining_pack =
                configuration.pack_set.for_file(absolute_path_of_definition);

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

        let referencing_pack = configuration
            .pack_set
            .for_file(referencing_file_path)
            .unwrap_or_else(|| {
                panic!(
                    "Could not find pack for referencing file path: {}",
                    &referencing_file_path.display()
                )
            });

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

pub(crate) trait ValidatorInterface {
    fn validate(&self, configuration: &Configuration) -> Option<String>;
}

// TODO: Break this function up into smaller functions
pub(crate) fn check_all(
    configuration: Configuration,
    files: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let initialized_dir =
        create_cache_dir_idempotently(&configuration.cache_directory);

    let cache = configuration.get_cache(initialized_dir);

    debug!("Intersecting input files with configuration included files");
    let absolute_paths: HashSet<PathBuf> = configuration.intersect_files(files);

    let violations: Vec<Violation> = get_all_violations(
        &configuration,
        absolute_paths,
        cache,
        configuration.experimental_parser,
    );
    let recorded_violations = &configuration.pack_set.all_violations;

    debug!("Filtering out recorded violations");
    let unrecorded_violations = violations
        .iter()
        .filter(|v| !recorded_violations.contains(&v.identifier))
        .collect::<Vec<&Violation>>();

    debug!("Finished filtering out recorded violations");

    let mut errors_present = false;

    if !unrecorded_violations.is_empty() {
        for violation in unrecorded_violations.iter() {
            println!("{}\n", violation.message);
        }

        println!("{} violation(s) detected:", unrecorded_violations.len());

        errors_present = true;
    }

    let validation_errors = validate(&configuration);
    if !validation_errors.is_empty() {
        errors_present = true;

        println!("{} validation error(s) detected:", validation_errors.len());
        for validation_error in validation_errors.iter() {
            println!("{}\n", validation_error);
        }
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
pub(crate) fn update(
    configuration: Configuration,
) -> Result<(), Box<dyn std::error::Error>> {
    let initialized_dir =
        create_cache_dir_idempotently(&configuration.cache_directory);
    let cache = configuration.get_cache(initialized_dir);

    let violations = get_all_violations(
        &configuration,
        configuration.intersect_files(vec![]),
        cache,
        configuration.experimental_parser,
    );

    package_todo::write_violations_to_disk(configuration, violations);
    println!("Successfully updated package_todo.yml files!");
    Ok(())
}

fn get_all_violations(
    configuration: &Configuration,
    absolute_paths: HashSet<PathBuf>,
    cache: Box<dyn Cache + Send + Sync>,
    experimental_parser: bool,
) -> Vec<Violation> {
    debug!("Getting unresolved references (using cache if possible)");
    let processed_files: Vec<ProcessedFile> = process_files_with_cache(
        &configuration.absolute_root,
        &absolute_paths,
        cache,
        experimental_parser,
    );

    let constant_resolver = if configuration.experimental_parser {
        get_experimental_constant_resolver(&processed_files)
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

    debug!("Finished turning unresolved references into fully qualified references");

    debug!("Running checkers on resolved references");
    let checkers: Vec<Box<dyn CheckerInterface + Send + Sync>> = vec![
        Box::new(dependency::Checker {}),
        Box::new(privacy::Checker {}),
        Box::new(visibility::Checker {}),
        Box::new(architecture::Checker {
            layers: configuration.layers.clone(),
        }),
    ];

    let violations: Vec<Violation> = checkers
        .into_par_iter()
        .flat_map(|c| {
            references
                .par_iter()
                .flat_map(|r| c.check(r))
                .collect::<Vec<Violation>>()
        })
        .collect();

    debug!("Finished running checkers");

    violations
}
