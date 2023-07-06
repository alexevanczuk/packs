use std::collections::HashMap;

use super::{CheckerInterface, ViolationIdentifier};
use crate::packs::checker::Reference;
use crate::packs::pack::Pack;
use crate::packs::Violation;
use petgraph::algo::tarjan_scc;
use petgraph::prelude::DiGraph;

pub struct Checker {}
impl Checker {
    fn validate(
        &self,
        configuration: crate::packs::Configuration,
    ) -> Option<String> {
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

        for pack in &configuration.pack_set.packs {
            for dependency_pack_name in &pack.dependencies {
                let from_pack = pack;
                let to_pack =
                    configuration.pack_set.for_pack(&dependency_pack_name);
                let from_node = pack_to_node
                    .get(&from_pack)
                    .expect("Could not find from_pack")
                    .to_owned();
                let to_node = pack_to_node
                    .get(&to_pack)
                    .expect("Could not find to_pack")
                    .to_owned();
                graph.add_edge(from_node, to_node, ());
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

        if sccs.is_empty() {
            None
        } else {
            let sccs_display = sccs.join("\n");

            let error_message = format!(
                "
Found {} strongly connected components (i.e. dependency cycles)
The following groups of packages from a cycle:

{}",
                sccs.len(),
                sccs_display
            );
            Some(error_message)
        }
    }
}

// TODO: Add test for ignored_dependencies
// Add test for does not enforce dependencies
impl CheckerInterface for Checker {
    fn check(&self, reference: &Reference) -> Option<Violation> {
        let referencing_pack = reference.referencing_pack;
        if referencing_pack.enforce_dependencies.is_false() {
            return None;
        }

        let referencing_pack_name = &referencing_pack.name;
        let defining_pack = &reference.defining_pack;

        if defining_pack.is_none() {
            return None;
        }

        let defining_pack = defining_pack.unwrap();

        let defining_pack_name = &defining_pack.name;
        if referencing_pack_name == defining_pack_name {
            return None;
        }

        let referencing_pack_dependencies = &referencing_pack.dependencies;

        let ignored_dependency = referencing_pack
            .ignored_dependencies
            .contains(defining_pack_name);
        if !referencing_pack_dependencies.contains(defining_pack_name)
            && !ignored_dependency
        {
            // START: Original packwerk message
            // path/to/file.rb:36:0
            // Dependency violation: ::Constant belongs to 'packs/defining_pack', but 'packs/referencing_pack/package.yml' does not specify a dependency on 'packs/defining_pack'.
            // Are we missing an abstraction?
            // Is the code making the reference, and the referenced constant, in the right packages?

            // Inference details: this is a reference to ::Constant which seems to be defined in packs/defining_pack/path/to/definition.rb.
            // To receive help interpreting or resolving this error message, see: https://github.com/Shopify/packwerk/blob/main/TROUBLESHOOT.md#Troubleshooting-violations
            // END: Original packwerk message

            let message = format!(
                "{}:{}:{}\nDependency violation: `{}` belongs to `{}`, but `{}` does not specify a dependency on `{}`.",
                reference.relative_referencing_file,
                reference.source_location.line,
                reference.source_location.column,
                reference.constant_name,
                defining_pack_name,
                referencing_pack.relative_yml().to_string_lossy(),
                defining_pack_name,
            );

            let violation_type = String::from("dependency");
            let file = reference.relative_referencing_file.clone();
            let identifier = ViolationIdentifier {
                violation_type,
                file,
                constant_name: reference.constant_name.clone(),
                referencing_pack_name: referencing_pack_name.clone(),
                defining_pack_name: defining_pack_name.clone(),
            };

            return Some(Violation {
                message,
                identifier,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs::*;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn referencing_and_defining_pack_are_identical() {
        let checker = Checker {};
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/simple_app")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
        );
        let reference = Reference {
            constant_name: String::from("::Foo"),
            defining_pack: Some(
                configuration.pack_set.for_pack(&String::from("packs/foo")),
            ),
            referencing_pack: configuration
                .pack_set
                .for_pack(&String::from("packs/foo")),
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/bar/app/services/bar.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };
        assert_eq!(None, checker.check(&reference))
    }

    #[test]
    fn test_check() {
        let checker = Checker {};
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/simple_app")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
        );
        let reference = Reference {
            constant_name: String::from("::Bar"),
            defining_pack: Some(
                configuration.pack_set.for_pack(&String::from("packs/bar")),
            ),
            referencing_pack: configuration
                .pack_set
                .for_pack(&String::from("packs/foo")),
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/bar/app/services/bar.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        let expected_violation = Violation {
            message: String::from("packs/foo/app/services/foo.rb:3:1\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."),
            identifier: ViolationIdentifier {
                violation_type: String::from("dependency"),
                file: String::from("packs/foo/app/services/foo.rb"),
                constant_name: String::from("::Bar"),
                referencing_pack_name: String::from("packs/foo"),
                defining_pack_name: String::from("packs/bar"),
            },
        };
        assert_eq!(expected_violation, checker.check(&reference).unwrap())
    }

    #[test]
    fn test_validate_with_cycle() {
        let checker = Checker {};
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/app_with_dependency_cycles")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
        );

        let error = checker.validate(configuration);
        let expected_message = String::from(
            "
Found 1 strongly connected components (i.e. dependency cycles)
The following groups of packages from a cycle:

packs/foo, packs/bar",
        );
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
        );

        let error = checker.validate(configuration);
        assert_eq!(error, None);
    }
}
