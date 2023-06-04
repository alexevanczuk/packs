use crate::packs::Configuration;
use rayon::{iter::ParallelBridge, prelude::ParallelIterator};
use std::{collections::HashSet, path::PathBuf};

struct Violation {
    message: String,
}
pub(crate) fn check(configuration: Configuration) {
    let absolute_paths: HashSet<PathBuf> =
        configuration.intersect_files(vec![]);
    let violations: Vec<Violation> = absolute_paths
        .into_iter()
        .par_bridge()
        .flat_map(get_violations)
        .collect();

    if !violations.is_empty() {
        println!("{} violation(s) detected:", violations.len());
        for violation in violations {
            println!("{}", violation.message);
        }
        std::process::exit(1);
    }
}

fn get_violations(_file: PathBuf) -> Vec<Violation> {
    vec![]
}
