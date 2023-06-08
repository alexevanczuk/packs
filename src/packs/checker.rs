use crate::packs::Configuration;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::ParallelIterator;
use std::path::Path;
use std::{collections::HashSet, path::PathBuf};
use tracing::debug;

use crate::packs;
use crate::packs::cache::get_unresolved_references;
use crate::packs::SourceLocation;

use super::UnresolvedReference;

pub mod dependency;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Violation {
    message: String,
}

#[derive(Debug)]
pub struct Reference {
    // We may later want to extract out a `Constant` struct
    constant_name: String,
    // Sometimes we cannot find the pack that a constant is defined in.
    // In this case, we return None, and do not include it.
    defining_pack_name: Option<String>,
    // We always know where the referencing file is, so we don't need an Option
    referencing_pack_name: String,
    relative_referencing_file: String,
    source_location: SourceLocation,
}
impl Reference {
    #[allow(unused_variables)]
    fn from_unresolved_reference(
        configuration: &Configuration,
        unresolved_reference: &UnresolvedReference,
        referencing_file_path: &Path,
    ) -> Reference {
        // Here we need to get a ConstantResolver from configuration
        // to figure out what package things are from.
        // We also need to implement Packs for_file.
        let maybe_constant = configuration.constant_resolver.resolve(
            &unresolved_reference.name,
            &unresolved_reference.namespace_path,
        );

        let defining_pack_name = if let Some(constant) = &maybe_constant {
            packs::for_file(
                configuration,
                &constant.absolute_path_of_definition,
            )
        } else {
            None
        };

        let constant_name = if let Some(constant) = &maybe_constant {
            &constant.fully_qualified_name
        } else {
            // Contant name is not known, so we'll just use the unresolved name for now
            &unresolved_reference.name
        };

        let constant_name = constant_name.clone();

        let referencing_pack_name =
            packs::for_file(configuration, referencing_file_path)
                .unwrap_or_else(|| {
                    panic!(
                        "Could not find pack for refrencing file path: {}",
                        &referencing_file_path.display()
                    )
                });

        let loc = unresolved_reference.location;
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
            defining_pack_name,
            referencing_pack_name,
            source_location,
            relative_referencing_file,
        }
    }
}

pub(crate) fn check(
    configuration: Configuration,
    files: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Interecting input files with configuration included files");
    let absolute_paths: HashSet<PathBuf> = configuration.intersect_files(files);

    // 1) Get the Vec<UnresolvedReferences> for each file in parallel
    // - Need a way for cache to do this, e.g. get_references_with_cache
    // 2) Turn those into a Vec<Reference>
    debug!("Getting unresolved references (using cache if possible)");
    let unresolved_references: Vec<(PathBuf, Vec<UnresolvedReference>)> =
        absolute_paths
            .into_par_iter()
            .map(|p| (p.clone(), get_unresolved_references(&configuration, &p)))
            .collect();
    let mut references: Vec<Reference> = vec![];

    debug!("Turning unresolved references into fully qualified references");
    for (absolute_path_of_referring_file, unresolved_refs) in
        unresolved_references
    {
        for unresolved_ref in unresolved_refs {
            references.push(Reference::from_unresolved_reference(
                &configuration,
                &unresolved_ref,
                &absolute_path_of_referring_file,
            ));
        }
    }

    debug!("Running checkers on resolved references");
    let checkers = vec![dependency::Checker {}];
    let violations: Vec<Violation> = references
        .into_par_iter()
        .flat_map(|r| {
            checkers
                .iter()
                .flat_map(|c| c.check(&configuration, &r))
                .collect::<Vec<Violation>>()
        })
        .collect();
    debug!("Finished running checkers");

    if !violations.is_empty() {
        println!("{} violation(s) detected:", violations.len());
        for violation in violations {
            println!("{}", violation.message);
        }
        Err("Violations detected".into())
    } else {
        println!("No violations detected");
        Ok(())
    }
}
