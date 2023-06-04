use crate::packs::Configuration;
use rayon::{iter::ParallelBridge, prelude::ParallelIterator};
use std::{collections::HashSet, path::PathBuf};

use crate::packs::SourceLocation;

use crate::packs::cache::get_unresolved_references;

use super::UnresolvedReference;

pub mod dependency;
pub struct Violation {
    message: String,
}

#[allow(dead_code)]
pub struct Constant {
    name: String,
    defining_pack_name: String,
}
#[allow(dead_code)]
pub struct Reference {
    constant: Constant,
    referencing_pack_name: String,
    source_location: SourceLocation,
}
pub(crate) fn check(configuration: Configuration) {
    let absolute_paths: HashSet<PathBuf> =
        configuration.intersect_files(vec![]);

    // 1) Get the Vec<UnresolvedReferences> for each file in parallel
    // - Need a way for cache to do this, e.g. get_references_with_cache
    // 2) Turn those into a Vec<Reference>
    let _unresolved_references: Vec<UnresolvedReference> = absolute_paths
        .into_iter()
        .par_bridge()
        .flat_map(|p| get_unresolved_references(&configuration, &p))
        .collect();

    // let references = ...
    let violations: Vec<Violation> = vec![];

    if !violations.is_empty() {
        println!("{} violation(s) detected:", violations.len());
        for violation in violations {
            println!("{}", violation.message);
        }
        std::process::exit(1);
    }
}
