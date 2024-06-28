use std::collections::HashMap;

use super::output_helper::print_reference_location;
use super::pack_checker::PackChecker;
use super::{CheckerInterface, ValidatorInterface};
use crate::packs::checker::Reference;
use crate::packs::pack::Pack;
use crate::packs::{Configuration, Violation};
use anyhow::Context;
use petgraph::algo::tarjan_scc;
use petgraph::prelude::DiGraph;

pub struct Checker {}
impl ValidatorInterface for Checker {
    fn validate(&self, configuration: &Configuration) -> Option<Vec<String>> {
        // configuration.pack_set
        let mut graph = DiGraph::<(), ()>::new();
        let mut pack_to_node: HashMap<&Pack, petgraph::prelude::NodeIndex> =
            HashMap::new();
        let mut node_to_pack: HashMap<petgraph::prelude::NodeIndex, &Pack> =
            HashMap::new();
        for pack in &configuration.pack_set.packs {
            let node = graph.add_node(());
            pack_to_node.insert(pack, node);
            node_to_pack.insert(node, pack);
        }

        let mut add_edge = |from_pack: &Pack, to_pack: &Pack| {
            let from_node = pack_to_node
                .get(&from_pack)
                .expect("Could not find from_pack")
                .to_owned();
            let to_node = pack_to_node
                .get(&to_pack)
                .expect("Could not find to_pack")
                .to_owned();
            graph.add_edge(from_node, to_node, ());
        };
        let mut error_messages: Vec<String> = vec![];

        match configuration.pack_set.all_pack_dependencies(configuration) {
            Ok(pack_dependencies) => {
                for pack_dependency in pack_dependencies {
                    if pack_dependency.from_pack == pack_dependency.to_pack {
                        error_messages.push(format!(
                            "Package cannot list itself as a dependency: {}",
                            pack_dependency
                                .from_pack
                                .relative_yml()
                                .to_string_lossy()
                        ));
                    } else {
                        add_edge(
                            pack_dependency.from_pack,
                            pack_dependency.to_pack,
                        );
                    }
                }
            }
            Err(msg) => {
                error_messages.push(msg.to_string());
                return Some(error_messages);
            }
        }

        let mut sccs = vec![];
        let strongly_componented_components = tarjan_scc(&graph);
        for component in strongly_componented_components {
            if component.len() > 1 {
                let pack_names: Vec<String> = component
                    .iter()
                    .map(|node_index| {
                        let pack = node_to_pack
                            .get(node_index)
                            .expect("Could not find pack name for node index");
                        pack.name.to_owned()
                    })
                    .collect();
                sccs.push(pack_names.join(", "));
            }
        }

        if !sccs.is_empty() {
            let sccs_display = sccs.join("\n\n");

            let error_message = format!(
                "
Found {} strongly connected components (i.e. dependency cycles)
The following groups of packages form a cycle:

{}",
                sccs.len(),
                sccs_display
            );
            error_messages.push(error_message);
        }

        if error_messages.is_empty() {
            None
        } else {
            Some(error_messages)
        }
    }
}

// TODO: Add test for does not enforce dependencies
impl CheckerInterface for Checker {
    fn check(
        &self,
        reference: &Reference,
        configuration: &Configuration,
    ) -> anyhow::Result<Option<Violation>> {
        let pack_checker =
            PackChecker::new(configuration, reference, &self.violation_type())?;
        if !pack_checker.checkable()? {
            return Ok(None);
        }
        let defining_pack = pack_checker.defining_pack.unwrap();

        let referencing_pack_dependencies =
            &pack_checker.referencing_pack.dependencies;

        let ignored_dependency = pack_checker
            .referencing_pack
            .ignored_dependencies
            .contains(&defining_pack.name);

        if referencing_pack_dependencies.contains(&defining_pack.name)
            || ignored_dependency
        {
            return Ok(None);
        }

        let relative_defining_file =
            reference.relative_defining_file.as_ref().context(format!(
                "expected a relative defining file for defining pack: {}",
                defining_pack.name
            ))?;

        if pack_checker
            .referencing_pack
            .is_ignored(relative_defining_file, &self.violation_type())?
        {
            return Ok(None);
        }

        // START: Original packwerk message
        // path/to/file.rb:36:0
        // Dependency violation: ::Constant belongs to 'packs/defining_pack', but 'packs/referencing_pack/package.yml' does not specify a dependency on 'packs/defining_pack'.
        // Are we missing an abstraction?
        // Is the code making the reference, and the referenced constant, in the right packages?

        // Inference details: this is a reference to ::Constant which seems to be defined in packs/defining_pack/path/to/definition.rb.
        // To receive help interpreting or resolving this error message, see: https://github.com/Shopify/packwerk/blob/main/TROUBLESHOOT.md#Troubleshooting-violations
        // END: Original packwerk message

        let loc = print_reference_location(reference);
        let message = format!(
                "{}Dependency violation: `{}` belongs to `{}`, but `{}` does not specify a dependency on `{}`.",
                loc,
                reference.constant_name,
                defining_pack.name,
                pack_checker.referencing_pack.relative_yml().to_string_lossy(),
                defining_pack.name,
            );

        Ok(Some(Violation {
            message,
            identifier: pack_checker.violation_identifier(),
        }))
    }

    fn violation_type(&self) -> String {
        "dependency".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use self::packs::{
        checker::common_test::tests::{
            build_expected_violation, default_defining_pack,
            default_referencing_pack, test_check, TestChecker,
        },
        pack::{CheckerSetting, EnforcementGlobsIgnore},
    };

    use super::*;
    use crate::packs::*;
    use pretty_assertions::assert_eq;
    use std::{collections::HashSet, path::PathBuf};

    #[test]
    fn test_reference_and_defining_packs_are_identical() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                ..default_defining_pack()
            }),
            referencing_pack: Pack {
                name: "packs/bar".to_owned(),
                relative_path: PathBuf::from("packs/bar"),
                enforce_dependencies: Some(CheckerSetting::True),
                ..default_referencing_pack()
            },
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_with_violation() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                ..default_defining_pack()
            }),
            referencing_pack: Pack{
                relative_path: PathBuf::from("packs/foo"),
                enforce_dependencies: Some(CheckerSetting::True),
                ..default_referencing_pack()},
            expected_violation: Some(build_expected_violation(
                "packs/foo/app/services/foo.rb:3:1\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`.".to_string(),
                "dependency".to_string(), false)),
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_with_strict_violation() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                ..default_defining_pack()
            }),
            referencing_pack: Pack{
                relative_path: PathBuf::from("packs/foo"),
                enforce_dependencies: Some(CheckerSetting::Strict),
                ..default_referencing_pack()},
            expected_violation: Some(build_expected_violation(
                "packs/foo/app/services/foo.rb:3:1\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`.".to_string(),
                "dependency".to_string(), true)),
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_ignored_dependency() -> anyhow::Result<()> {
        let mut ignored_dependencies = HashSet::new();
        ignored_dependencies.insert(String::from("packs/bar"));

        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                ..default_defining_pack()
            }),
            referencing_pack: Pack {
                relative_path: PathBuf::from("packs/foo"),
                ignored_dependencies,
                enforce_dependencies: Some(CheckerSetting::True),
                ..default_referencing_pack()
            },
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_with_enforcement_globs_ignore() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                ..default_defining_pack()
            }),
            referencing_pack: Pack {
                relative_path: PathBuf::from("packs/foo"),
                enforce_dependencies: Some(CheckerSetting::True),
                enforcement_globs_ignore: Some(vec![EnforcementGlobsIgnore {
                    enforcements: ["dependency"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    ignores: ["packs/bar/**"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    reason: "deprecated".to_string(),
                }]),
                ..default_referencing_pack()
            },
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_validate_with_cycle() {
        let checker = Checker {};
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/app_with_dependency_cycles")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
        )
        .unwrap();

        let error = checker.validate(&configuration);
        let expected_message = vec![String::from("Package cannot list itself as a dependency: packs/baz/package.yml"),
            String::from(
            "
Found 1 strongly connected components (i.e. dependency cycles)
The following groups of packages form a cycle:

packs/foo, packs/bar",
        )];
        assert_eq!(error, Some(expected_message));
    }

    #[test]
    fn test_validate_without_cycle() {
        let checker = Checker {};
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/simple_app")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
        )
        .unwrap();

        let error = checker.validate(&configuration);
        assert_eq!(error, None);
    }

    #[test]
    #[should_panic(
        expected = "tests/fixtures/contains_duplicates_in_package/packs/bar/package.yml"
    )]
    fn test_invalid_package_yml() {
        let checker = Checker {};
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/contains_duplicates_in_package")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
        )
        .unwrap();

        checker.validate(&configuration);
    }
}
