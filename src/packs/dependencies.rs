use std::collections::HashMap;

use super::Configuration;

type PackName = String;
type ViolationType = String;
type ViolationCount = usize;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Dependencies {
    pub explicit: Vec<PackName>,
    pub implicit: HashMap<PackName, HashMap<ViolationType, ViolationCount>>,
}

pub fn find_dependencies(
    configuration: &Configuration,
    pack_name: &str,
) -> anyhow::Result<Dependencies> {
    let pack = configuration.pack_set.for_pack(pack_name)?;

    let mut public_dependencies: Vec<PackName> = configuration
        .pack_set
        .packs
        .iter()
        .filter(|p| p.name != pack.name && p.dependencies.contains(&pack.name))
        .map(|p| p.name.clone())
        .collect();
    public_dependencies.sort();

    let mut implicit_dependencies: HashMap<
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
                        let entry = implicit_dependencies
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

    Ok(Dependencies {
        explicit: public_dependencies,
        implicit: implicit_dependencies,
    })
}

#[cfg(test)]
mod tests {
    use crate::packs::configuration;

    use super::*;
    use std::path::PathBuf;

    #[test]
    fn find_explicit_dependencies() {
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/simple_app")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
        )
        .unwrap();

        let dependencies =
            find_dependencies(&configuration, "packs/baz").unwrap();
        assert_eq!(dependencies.explicit.len(), 1);
        assert!(dependencies.explicit.contains(&String::from("packs/foo")));
        assert_eq!(dependencies.implicit.len(), 0);
    }

    #[test]
    fn find_implicit_dependencies() {
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/contains_package_todo")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
        )
        .unwrap();

        let dependencies =
            find_dependencies(&configuration, "packs/bar").unwrap();
        assert_eq!(dependencies.explicit.len(), 0);
        assert_eq!(dependencies.implicit.len(), 1);
        assert_eq!(dependencies.implicit.get("packs/foo").unwrap().len(), 1);
        assert_eq!(
            dependencies
                .implicit
                .get("packs/foo")
                .unwrap()
                .get("dependency")
                .unwrap(),
            &1usize
        );
    }
}
