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

    let public_dependents: HashSet<PackName> = configuration
        .pack_set
        .packs
        .iter()
        .filter(|p| {
            dbg!(&p.name);
            dbg!(&p.dependencies);
            p.name != pack.name && p.dependencies.contains(&pack.name)
        })
        .map(|p| p.name.clone())
        .collect();

    let mut violation_dependents: HashMap<
        PackName,
        HashMap<ViolationType, ViolationCount>,
    > = HashMap::new();

    for current_pack in &configuration.pack_set.packs {
        if current_pack.name != pack.name {
            for (violation_pack_name, violation_groups) in
                &current_pack.package_todo.violations_by_defining_pack
            {
                if violation_pack_name == &pack.name {
                    for violation_group in violation_groups.values() {
                        let entry = violation_dependents
                            .entry(current_pack.name.clone())
                            .or_default();
                        for violation_type in &violation_group.violation_types {
                            entry
                                .entry(violation_type.clone())
                                .and_modify(|e| *e += 1)
                                .or_insert(1);
                        }
                    }
                }
            }
        }
    }

    Ok(Dependents {
        public_dependents,
        violation_dependents,
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
        assert_eq!(dependents.violation_dependents.len(), 0);
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
        assert_eq!(
            dependents
                .violation_dependents
                .get("packs/foo")
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            dependents
                .violation_dependents
                .get("packs/foo")
                .unwrap()
                .get("dependency")
                .unwrap(),
            &1usize
        );
    }
}
