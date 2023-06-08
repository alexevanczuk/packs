use crate::packs::Configuration;
use rayon::{iter::ParallelBridge, prelude::ParallelIterator};
use std::{collections::HashSet, path::PathBuf};

use crate::packs::SourceLocation;

use crate::packs::cache::get_unresolved_references;

use super::UnresolvedReference;

pub mod dependency;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Violation {
    message: String,
}

#[allow(dead_code)]
pub struct Reference {
    // We may later want to extract out a `Constant` struct
    constant_name: String,
    defining_pack_name: String,
    referencing_pack_name: String,
    source_location: SourceLocation,
}
impl Reference {
    #[allow(unused_variables)]
    fn from_unresolved_reference(
        configuration: &Configuration,
        r: &UnresolvedReference,
    ) -> Reference {
        // Here we need to get a ConstantResolver from configuration
        // to figure out what package things are from.
        // We also need to implement Packs for_file.
        todo!()
    }
}
pub(crate) fn check(configuration: Configuration) {
    let absolute_paths: HashSet<PathBuf> =
        configuration.intersect_files(vec![]);

    // 1) Get the Vec<UnresolvedReferences> for each file in parallel
    // - Need a way for cache to do this, e.g. get_references_with_cache
    // 2) Turn those into a Vec<Reference>
    let unresolved_references: Vec<UnresolvedReference> = absolute_paths
        .into_iter()
        .par_bridge()
        .flat_map(|p| get_unresolved_references(&configuration, &p))
        .collect();

    let references: Vec<Reference> = unresolved_references
        .into_iter()
        .map(|r| Reference::from_unresolved_reference(&configuration, &r))
        .collect();

    let checkers = vec![dependency::Checker {}];
    let violations: Vec<Violation> = references
        .into_iter()
        .par_bridge()
        .flat_map(|r| {
            checkers
                .iter()
                .flat_map(|c| c.check(&configuration, &r))
                .collect::<Vec<Violation>>()
        })
        .collect();

    if !violations.is_empty() {
        println!("{} violation(s) detected:", violations.len());
        for violation in violations {
            println!("{}", violation.message);
        }
        std::process::exit(1);
    }
}
