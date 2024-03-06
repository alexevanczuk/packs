use std::collections::{HashMap, HashSet};

use super::Configuration;
use std::fmt::Display;

type PackName = String;
type ViolationType = String;
type ViolationCount = usize;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Dependents {
    public_dependents: HashSet<PackName>,
    violation_dependents:
        HashMap<PackName, HashMap<ViolationType, ViolationCount>>,
}

impl Display for Dependents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dependents")
    }
}

pub fn find_dependents(
    configuration: &Configuration,
    pack_name: &str,
) -> anyhow::Result<Dependents> {
    let pack = configuration.pack_set.for_pack(pack_name)?;

    let mut public_dependents: HashSet<PackName> = configuration
        .pack_set
        .packs
        .iter()
        .filter(|p| {
            dbg!(&p.name);
            dbg!(&p.dependencies);
            &p.name != &pack.name && p.dependencies.contains(&pack.name)
        })
        .map(|p| p.name.clone())
        .collect();

    Ok(Dependents {
        public_dependents,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use crate::packs::configuration;

    use super::*;
    use std::path::PathBuf;

    #[test]
    fn find_dependents_with_violations() {
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/simple_app")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
        )
        .unwrap();

        let dependents = find_dependents(&configuration, "packs/baz").unwrap();
        assert_eq!(dependents.public_dependents.len(), 1);
        assert!(dependents.public_dependents.contains("packs/foo"));
    }

    #[test]
    fn find_dependents_without_violations() {
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/contains_package_todo")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
        )
        .unwrap();

        let dependents = find_dependents(&configuration, "packs/bar").unwrap();
        assert_eq!(dependents.public_dependents.len(), 0);
        assert_eq!(dependents.violation_dependents.len(), 1);
    }
}
