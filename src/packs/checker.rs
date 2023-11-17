// Module declarations
pub(crate) mod architecture;
mod dependency;
mod privacy;
pub(crate) mod reference;
mod visibility;

// Internal imports
use crate::packs::pack::write_pack_to_disk;
use crate::packs::pack::Pack;
use crate::packs::package_todo;
use crate::packs::Configuration;
use crate::packs::PackSet;

// External imports
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use reference::Reference;
use std::collections::HashMap;
use std::{collections::HashSet, path::PathBuf};
use tracing::debug;

use super::reference_extractor::get_all_references;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct ViolationIdentifier {
    pub violation_type: String,
    pub file: String,
    pub constant_name: String,
    pub referencing_pack_name: String,
    pub defining_pack_name: String,
}

pub fn get_defining_pack<'a>(
    violation: &ViolationIdentifier,
    packset: &'a PackSet,
) -> &'a Pack {
    let defining_pack_result = packset.for_pack(&violation.defining_pack_name);

    // For some reason cargo fmt wasn't formatting this closure correctly when passed directly into unwrap_or_else
    let error_closure = |_| {
        let error_message = format!(
            "ViolationIdentifier#defining_pack is {}, but that pack cannot be found in the packset.",
            &violation.defining_pack_name
        );

        panic!("{}", &error_message);
    };

    defining_pack_result.unwrap_or_else(error_closure)
}

pub fn get_referencing_pack<'a>(
    violation: &ViolationIdentifier,
    packset: &'a PackSet,
) -> &'a Pack {
    let referencing_pack_result =
        packset.for_pack(&violation.referencing_pack_name);

    // For some reason cargo fmt wasn't formatting this closure correctly when passed directly into unwrap_or_else
    let error_closure = |_| {
        let error_message = format!(
            "ViolationIdentifier#referencing_pack is {}, but that pack cannot be found in the packset.",
            &violation.referencing_pack_name
        );

        panic!("{}", &error_message);
    };

    referencing_pack_result.unwrap_or_else(error_closure)
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

    fn is_strict_mode_violation(
        &self,
        offense: &ViolationIdentifier,
        configuration: &Configuration,
    ) -> bool;

    fn violation_type(&self) -> String;
}

pub(crate) trait ValidatorInterface {
    fn validate(&self, configuration: &Configuration) -> Option<String>;
}

#[derive(Debug, PartialEq)]
struct CheckAll<'a> {
    reportable_violations: HashSet<&'a Violation>,
    stale_violations: Vec<&'a ViolationIdentifier>,
    strict_mode_violations: Vec<&'a ViolationIdentifier>,
}
struct CheckAllBuilder<'a> {
    configuration: &'a Configuration,
    found_violations: &'a FoundViolations,
}

struct FoundViolations {
    checkers: Vec<Box<dyn CheckerInterface + Send + Sync>>,
    absolute_paths: HashSet<PathBuf>,
    violations: HashSet<Violation>,
}

impl<'a> CheckAllBuilder<'a> {
    fn new(
        configuration: &'a Configuration,
        found_violations: &'a FoundViolations,
    ) -> Self {
        Self {
            configuration,
            found_violations,
        }
    }

    pub fn build(mut self) -> CheckAll<'a> {
        let recorded_violations = &self.configuration.pack_set.all_violations;

        CheckAll {
            reportable_violations: self
                .build_reportable_violations(recorded_violations),
            stale_violations: self.build_stale_violations(recorded_violations),
            strict_mode_violations: self
                .build_strict_mode_violations(recorded_violations),
        }
    }

    fn build_reportable_violations(
        &mut self,
        recorded_violations: &HashSet<ViolationIdentifier>,
    ) -> HashSet<&'a Violation> {
        let reportable_violations =
            if self.configuration.ignore_recorded_violations {
                debug!("Filtering recorded violations is disabled in config");
                self.found_violations.violations.iter().collect()
            } else {
                self.found_violations
                    .violations
                    .iter()
                    .filter(|v| !recorded_violations.contains(&v.identifier))
                    .collect()
            };
        reportable_violations
    }

    fn build_stale_violations(
        &mut self,
        recorded_violations: &'a HashSet<ViolationIdentifier>,
    ) -> Vec<&'a ViolationIdentifier> {
        let found_violation_identifiers: HashSet<&ViolationIdentifier> = self
            .found_violations
            .violations
            .par_iter()
            .map(|v| &v.identifier)
            .collect();

        let relative_files = self
            .found_violations
            .absolute_paths
            .iter()
            .map(|p| {
                p.strip_prefix(&self.configuration.absolute_root)
                    .unwrap()
                    .to_str()
                    .unwrap()
            })
            .collect::<HashSet<&str>>();

        let stale_violations = recorded_violations
            .par_iter()
            .filter(|v_identifier| {
                relative_files.contains(&v_identifier.file.as_str())
                    && !found_violation_identifiers.contains(v_identifier)
            })
            .collect::<Vec<&ViolationIdentifier>>();
        stale_violations
    }

    fn build_strict_mode_violations(
        &mut self,
        recorded_violations: &'a HashSet<ViolationIdentifier>,
    ) -> Vec<&'a ViolationIdentifier> {
        let mut indexed_checkers: HashMap<
            String,
            &Box<dyn CheckerInterface + Send + Sync>,
        > = HashMap::new();
        for checker in &self.found_violations.checkers {
            indexed_checkers.insert(checker.violation_type(), checker);
        }

        let strict_mode_violations: Vec<&'a ViolationIdentifier> =
            recorded_violations
                .iter()
                .filter(|v| {
                    indexed_checkers
                        .get(&v.violation_type)
                        .unwrap()
                        .is_strict_mode_violation(v, self.configuration)
                })
                .collect();
        strict_mode_violations
    }
}
pub(crate) fn check_all(
    configuration: &Configuration,
    files: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let checkers = get_checkers(configuration);

    debug!("Intersecting input files with configuration included files");
    let absolute_paths: HashSet<PathBuf> =
        configuration.intersect_files(files.clone());

    let violations: HashSet<Violation> =
        get_all_violations(configuration, &absolute_paths, &checkers);
    let found_violations = FoundViolations {
        checkers,
        absolute_paths,
        violations,
    };
    let check_all =
        CheckAllBuilder::new(configuration, &found_violations).build();
    let mut errors_present = false;

    if !check_all.reportable_violations.is_empty() {
        for violation in check_all.reportable_violations.iter() {
            println!("{}\n", violation.message);
        }

        println!(
            "{} violation(s) detected:",
            check_all.reportable_violations.len()
        );

        errors_present = true;
    }

    if !check_all.stale_violations.is_empty() {
        println!(
            "There were stale violations found, please run `packs update`"
        );
        errors_present = true;
    }

    if !check_all.strict_mode_violations.is_empty() {
        for v in check_all.strict_mode_violations {
            let error_message = format!("{} cannot have {} violations on {} because strict mode is enabled for {} violations in the enforcing pack's package.yml file",
                                        v.referencing_pack_name,
                                        v.violation_type,
                                        v.defining_pack_name,
                                        v.violation_type
            );
            println!("{}", error_message);
        }

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
    configuration: &Configuration,
) -> Result<(), Box<dyn std::error::Error>> {
    let checkers = get_checkers(configuration);

    let violations = get_all_violations(
        configuration,
        &configuration.included_files,
        &checkers,
    );

    package_todo::write_violations_to_disk(configuration, violations);
    println!("Successfully updated package_todo.yml files!");
    Ok(())
}

pub(crate) fn remove_unnecessary_dependencies(
    configuration: &Configuration,
) -> Result<(), Box<dyn std::error::Error>> {
    let unnecessary_dependencies = get_unnecessary_dependencies(configuration);
    for (pack, dependency_names) in unnecessary_dependencies.iter() {
        remove_reference_to_dependency(pack, dependency_names);
    }
    Ok(())
}

pub(crate) fn check_unnecessary_dependencies(
    configuration: &Configuration,
) -> Result<(), Box<dyn std::error::Error>> {
    let unnecessary_dependencies = get_unnecessary_dependencies(configuration);
    if unnecessary_dependencies.is_empty() {
        Ok(())
    } else {
        for (pack, dependency_names) in unnecessary_dependencies.iter() {
            for dependency_name in dependency_names {
                println!(
                    "{} depends on {} but does not use it",
                    pack.name, dependency_name
                )
            }
        }
        Err("List unnecessary dependencies failed".into())
    }
}

fn get_unnecessary_dependencies(
    configuration: &Configuration,
) -> HashMap<Pack, Vec<String>> {
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

    let mut unnecessary_dependencies: HashMap<Pack, Vec<String>> =
        HashMap::new();
    for pack in &configuration.pack_set.packs {
        for dependency_name in &pack.dependencies {
            let edge_key = (pack.name.clone(), dependency_name.clone());
            let edge_count = edge_counts.get(&edge_key).unwrap_or(&0);
            if edge_count == &0 {
                unnecessary_dependencies
                    .entry(pack.clone())
                    .or_default()
                    .push(dependency_name.clone());
            }
        }
    }

    unnecessary_dependencies
}

fn get_all_violations(
    configuration: &Configuration,
    absolute_paths: &HashSet<PathBuf>,
    checkers: &Vec<Box<dyn CheckerInterface + Send + Sync>>,
) -> HashSet<Violation> {
    let references = get_all_references(configuration, absolute_paths);

    debug!("Running checkers on resolved references");

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

fn get_checkers(
    configuration: &Configuration,
) -> Vec<Box<dyn CheckerInterface + Send + Sync>> {
    vec![
        Box::new(dependency::Checker {}),
        Box::new(privacy::Checker {}),
        Box::new(visibility::Checker {}),
        Box::new(architecture::Checker {
            layers: configuration.layers.clone(),
        }),
    ]
}

fn remove_reference_to_dependency(pack: &Pack, dependency_names: &[String]) {
    let without_dependency = pack
        .dependencies
        .iter()
        .filter(|dependency| !dependency_names.contains(dependency));
    let updated_pack = Pack {
        dependencies: without_dependency.cloned().collect(),
        ..pack.clone()
    };
    write_pack_to_disk(&updated_pack);
}
