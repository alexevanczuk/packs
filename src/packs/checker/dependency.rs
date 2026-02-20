use std::collections::{HashMap, HashSet, VecDeque};

use super::output_helper::print_reference_location;
use super::pack_checker::PackChecker;
use super::{CheckerInterface, CycleEdge, ValidationError, ValidatorInterface};
use crate::packs::checker::Reference;
use crate::packs::pack::Pack;
use crate::packs::{Configuration, Violation};
use anyhow::Context;
use petgraph::algo::tarjan_scc;
use petgraph::prelude::DiGraph;
use petgraph::Direction;

/// Build graph infrastructure shared by both validate and validate_structured.
struct DependencyGraph<'a> {
    graph: DiGraph<(), ()>,
    pack_to_node: HashMap<&'a Pack, petgraph::prelude::NodeIndex>,
    node_to_pack: HashMap<petgraph::prelude::NodeIndex, &'a Pack>,
}

fn build_dependency_graph<'a>(
    configuration: &'a Configuration,
) -> Result<(DependencyGraph<'a>, Vec<&'a Pack>), String> {
    let mut graph = DiGraph::<(), ()>::new();
    let mut pack_to_node: HashMap<&Pack, petgraph::prelude::NodeIndex> =
        HashMap::new();
    let mut node_to_pack: HashMap<petgraph::prelude::NodeIndex, &Pack> =
        HashMap::new();
    let mut self_deps: Vec<&Pack> = vec![];

    for pack in &configuration.pack_set.packs {
        let node = graph.add_node(());
        pack_to_node.insert(pack, node);
        node_to_pack.insert(node, pack);
    }

    match configuration.pack_set.all_pack_dependencies(configuration) {
        Ok(pack_dependencies) => {
            for pack_dependency in pack_dependencies {
                if pack_dependency.from_pack == pack_dependency.to_pack {
                    self_deps.push(pack_dependency.from_pack);
                } else {
                    let from_node = pack_to_node
                        .get(&pack_dependency.from_pack)
                        .expect("Could not find from_pack")
                        .to_owned();
                    let to_node = pack_to_node
                        .get(&pack_dependency.to_pack)
                        .expect("Could not find to_pack")
                        .to_owned();
                    graph.add_edge(from_node, to_node, ());
                }
            }
        }
        Err(msg) => {
            return Err(msg.to_string());
        }
    }

    Ok((
        DependencyGraph {
            graph,
            pack_to_node,
            node_to_pack,
        },
        self_deps,
    ))
}

pub struct Checker {}
impl ValidatorInterface for Checker {
    fn validate(&self, configuration: &Configuration) -> Option<Vec<String>> {
        // Convert structured errors to strings for backward compatibility
        let structured = validate_structured(configuration);
        if structured.is_empty() {
            return None;
        }

        let mut error_messages: Vec<String> = vec![];

        // Collect configuration errors (e.g., unknown packs)
        for err in structured
            .iter()
            .filter(|e| e.error_type == "configuration")
        {
            error_messages.push(err.message.clone());
        }
        // If we have configuration errors, return early (matches old behavior)
        if !error_messages.is_empty() {
            return Some(error_messages);
        }

        // Collect self-dependency messages
        for err in structured
            .iter()
            .filter(|e| e.error_type == "self_dependency")
        {
            error_messages.push(err.message.clone());
        }

        // Collect cycle messages
        let cycle_errors: Vec<&ValidationError> = structured
            .iter()
            .filter(|e| e.error_type == "cycle")
            .collect();
        if !cycle_errors.is_empty() {
            let mut sccs: Vec<String> = vec![];
            for err in &cycle_errors {
                if let Some(edges) = &err.cycle_edges {
                    let path: Vec<String> = edges
                        .iter()
                        .map(|e| format!("{}/package.yml", e.from_pack))
                        .collect();
                    let mut display = path;
                    if let Some(first_edge) = edges.first() {
                        display.push(format!(
                            "{}/package.yml",
                            first_edge.from_pack
                        ));
                    }
                    sccs.push(display.join(" -> "));
                }
            }
            if !sccs.is_empty() {
                let sccs_display = sccs.join("\n\n");
                error_messages.push(format!(
                    "\nFound {} strongly connected components (i.e. dependency cycles)\nThe following groups of packages form a cycle:\n\n{}",
                    sccs.len(),
                    sccs_display
                ));
            }
        }

        // Collect strict_transitive messages
        for err in structured
            .iter()
            .filter(|e| e.error_type == "strict_transitive")
        {
            error_messages.push(err.message.clone());
        }

        if error_messages.is_empty() {
            None
        } else {
            Some(error_messages)
        }
    }
}

/// Returns structured validation errors for dependency-related checks.
pub fn validate_structured(
    configuration: &Configuration,
) -> Vec<ValidationError> {
    let mut errors: Vec<ValidationError> = vec![];

    let (dep_graph, self_deps) = match build_dependency_graph(configuration) {
        Ok(result) => result,
        Err(msg) => {
            errors.push(ValidationError {
                error_type: "configuration".to_string(),
                message: msg,
                cycle_edges: None,
                file: None,
            });
            return errors;
        }
    };

    // Self-dependency errors
    for pack in self_deps {
        let file = pack.relative_yml().to_string_lossy().to_string();
        errors.push(ValidationError {
            error_type: "self_dependency".to_string(),
            message: format!(
                "Package cannot list itself as a dependency: {}",
                file
            ),
            cycle_edges: None,
            file: Some(file),
        });
    }

    // Cycle detection
    let strongly_connected_components = tarjan_scc(&dep_graph.graph);
    for component in strongly_connected_components {
        if component.len() > 1 {
            let scc_nodes: HashSet<_> = component.iter().cloned().collect();
            if let Some(cycle_nodes) =
                find_cycle_in_scc_nodes(&scc_nodes, &dep_graph.graph)
            {
                // Build cycle edges from the node path
                let mut cycle_edges: Vec<CycleEdge> = vec![];
                for i in 0..cycle_nodes.len() {
                    let from_node = cycle_nodes[i];
                    let to_node = cycle_nodes[(i + 1) % cycle_nodes.len()];
                    let from_pack =
                        dep_graph.node_to_pack.get(&from_node).unwrap();
                    let to_pack = dep_graph.node_to_pack.get(&to_node).unwrap();
                    cycle_edges.push(CycleEdge {
                        from_pack: from_pack.name.clone(),
                        to_pack: to_pack.name.clone(),
                        file: from_pack
                            .relative_yml()
                            .to_string_lossy()
                            .to_string(),
                    });
                }

                let edge_display: Vec<String> = cycle_edges
                    .iter()
                    .map(|e| format!("{} -> {}", e.from_pack, e.to_pack))
                    .collect();

                errors.push(ValidationError {
                    error_type: "cycle".to_string(),
                    message: format!(
                        "Dependency cycle detected: {}",
                        edge_display.join(", ")
                    ),
                    cycle_edges: Some(cycle_edges),
                    file: None,
                });
            }
        }
    }

    // Strict transitive validation
    let strict_violations = find_strict_violations(
        &dep_graph.pack_to_node,
        &dep_graph.node_to_pack,
        &dep_graph.graph,
    );
    for (strict_pack_name, path) in strict_violations {
        let path_display = path.join(" -> ");
        errors.push(ValidationError {
            error_type: "strict_transitive".to_string(),
            message: format!(
                "{} has `enforce_dependencies: strict` but has a non-strict transitive dependency: {}",
                strict_pack_name, path_display
            ),
            cycle_edges: None,
            file: None,
        });
    }

    errors
}

/// Find a cycle path within a strongly connected component using DFS.
/// Returns the cycle as a Vec of NodeIndex values (without the closing node).
fn find_cycle_in_scc_nodes(
    scc_nodes: &HashSet<petgraph::prelude::NodeIndex>,
    graph: &DiGraph<(), ()>,
) -> Option<Vec<petgraph::prelude::NodeIndex>> {
    let start = *scc_nodes.iter().next()?;

    let mut visited: HashSet<petgraph::prelude::NodeIndex> = HashSet::new();
    let mut path: Vec<petgraph::prelude::NodeIndex> = vec![];

    fn dfs(
        current: petgraph::prelude::NodeIndex,
        start: petgraph::prelude::NodeIndex,
        scc_nodes: &HashSet<petgraph::prelude::NodeIndex>,
        visited: &mut HashSet<petgraph::prelude::NodeIndex>,
        path: &mut Vec<petgraph::prelude::NodeIndex>,
        graph: &DiGraph<(), ()>,
    ) -> bool {
        visited.insert(current);
        path.push(current);

        for neighbor in graph.neighbors(current) {
            if !scc_nodes.contains(&neighbor) {
                continue;
            }

            if neighbor == start && path.len() > 1 {
                return true;
            }

            if !visited.contains(&neighbor)
                && dfs(neighbor, start, scc_nodes, visited, path, graph)
            {
                return true;
            }
        }

        path.pop();
        false
    }

    if dfs(start, start, scc_nodes, &mut visited, &mut path, graph) {
        Some(path)
    } else {
        None
    }
}

/// Efficiently find all strict packs that transitively depend on non-strict packs.
/// Uses reverse BFS to identify violating packs in O(nodes + edges), then finds paths only for violations.
fn find_strict_violations<'a>(
    pack_to_node: &HashMap<&'a Pack, petgraph::prelude::NodeIndex>,
    node_to_pack: &HashMap<petgraph::prelude::NodeIndex, &'a Pack>,
    graph: &DiGraph<(), ()>,
) -> Vec<(String, Vec<String>)> {
    // Step 1: Identify non-strict and strict packs
    let mut non_strict_nodes: HashSet<petgraph::prelude::NodeIndex> =
        HashSet::new();
    let mut strict_nodes: HashSet<petgraph::prelude::NodeIndex> =
        HashSet::new();

    for (pack, &node) in pack_to_node {
        let is_strict = pack
            .enforce_dependencies
            .as_ref()
            .map_or(false, |s| s.is_strict());
        if is_strict {
            strict_nodes.insert(node);
        } else {
            non_strict_nodes.insert(node);
        }
    }

    if non_strict_nodes.is_empty() || strict_nodes.is_empty() {
        return Vec::new();
    }

    // Step 2: BFS from non-strict nodes using incoming edges to find all packs that can reach them
    // A pack "can reach" a non-strict pack if it depends on it (directly or transitively)
    // Using Incoming direction = following edges backwards = finding dependents
    let mut can_reach_non_strict: HashSet<petgraph::prelude::NodeIndex> =
        HashSet::new();
    let mut queue: VecDeque<petgraph::prelude::NodeIndex> = VecDeque::new();

    // Start BFS from all non-strict nodes
    for &node in &non_strict_nodes {
        can_reach_non_strict.insert(node);
        queue.push_back(node);
    }

    while let Some(current) = queue.pop_front() {
        // Incoming = nodes that have edges pointing TO current (i.e., nodes that depend on current)
        for neighbor in graph.neighbors_directed(current, Direction::Incoming) {
            if !can_reach_non_strict.contains(&neighbor) {
                can_reach_non_strict.insert(neighbor);
                queue.push_back(neighbor);
            }
        }
    }

    // Step 3: Find strict packs that can reach non-strict packs (these are violations)
    let violating_strict_nodes: Vec<petgraph::prelude::NodeIndex> =
        strict_nodes
            .iter()
            .filter(|node| can_reach_non_strict.contains(node))
            .copied()
            .collect();

    // Step 4: For each violation, find shortest path to a non-strict dependency (for error message)
    let mut results = Vec::new();
    for start_node in violating_strict_nodes {
        if let Some(path) = find_path_to_non_strict(
            start_node,
            &non_strict_nodes,
            node_to_pack,
            graph,
        ) {
            let start_pack = node_to_pack.get(&start_node).unwrap();
            results.push((start_pack.name.clone(), path));
        }
    }

    results
}

/// BFS to find shortest path from a node to any non-strict node
fn find_path_to_non_strict(
    start: petgraph::prelude::NodeIndex,
    non_strict_nodes: &HashSet<petgraph::prelude::NodeIndex>,
    node_to_pack: &HashMap<petgraph::prelude::NodeIndex, &Pack>,
    graph: &DiGraph<(), ()>,
) -> Option<Vec<String>> {
    let mut visited: HashSet<petgraph::prelude::NodeIndex> = HashSet::new();
    let mut queue: VecDeque<(petgraph::prelude::NodeIndex, Vec<String>)> =
        VecDeque::new();

    let start_pack = node_to_pack.get(&start)?;
    queue.push_back((start, vec![start_pack.name.clone()]));
    visited.insert(start);

    while let Some((current, path)) = queue.pop_front() {
        for neighbor in graph.neighbors(current) {
            if visited.contains(&neighbor) {
                continue;
            }
            visited.insert(neighbor);

            let neighbor_pack = node_to_pack.get(&neighbor)?;
            let mut new_path = path.clone();
            new_path.push(neighbor_pack.name.clone());

            if non_strict_nodes.contains(&neighbor) {
                return Some(new_path);
            }

            // Only continue through strict nodes
            let is_strict = neighbor_pack
                .enforce_dependencies
                .as_ref()
                .map_or(false, |s| s.is_strict());
            if is_strict {
                queue.push_back((neighbor, new_path));
            }
        }
    }

    None
}

// TODO: Add test for does not enforce dependencies
impl CheckerInterface for Checker {
    fn check(
        &self,
        reference: &Reference,
        configuration: &Configuration,
        _sigils: &HashMap<std::path::PathBuf, Vec<crate::packs::Sigil>>,
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
            source_location: reference.source_location.clone(),
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
            &1,
        )
        .unwrap();

        let error = checker.validate(&configuration);
        assert!(error.is_some());
        let errors = error.unwrap();
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().any(|e| e.contains(
            "Package cannot list itself as a dependency: packs/baz/package.yml"
        )));
        // The cycle should show the dependency path, e.g., "packs/bar -> packs/foo -> packs/bar"
        assert!(errors
            .iter()
            .any(|e| e.contains("strongly connected components")
                && e.contains(" -> ")));
    }

    #[test]
    fn test_validate_without_cycle() {
        let checker = Checker {};
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/simple_app")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
            &1,
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
            &1,
        )
        .unwrap();

        checker.validate(&configuration);
    }

    #[test]
    fn test_validate_strict_depends_on_non_strict() {
        let checker = Checker {};
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/strict_depends_on_non_strict")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
            &1,
        )
        .unwrap();

        let error = checker.validate(&configuration);
        let expected_message = vec![String::from(
            "packs/foo has `enforce_dependencies: strict` but has a non-strict transitive dependency: packs/foo -> packs/bar",
        )];
        assert_eq!(error, Some(expected_message));
    }

    #[test]
    fn test_validate_strict_transitive_non_strict() {
        let checker = Checker {};
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/strict_transitive_non_strict")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
            &1,
        )
        .unwrap();

        let error = checker.validate(&configuration);
        // Both foo and bar are strict and depend (transitively) on non-strict baz
        // foo -> bar -> baz (baz is non-strict)
        // bar -> baz (baz is non-strict)
        assert!(error.is_some());
        let errors = error.unwrap();
        assert_eq!(errors.len(), 2);
        assert!(errors
            .iter()
            .any(|e| e.contains("packs/foo") && e.contains("packs/baz")));
        assert!(errors
            .iter()
            .any(|e| e.contains("packs/bar") && e.contains("packs/baz")));
    }

    #[test]
    fn test_validate_strict_mode_no_violation() {
        // The existing uses_strict_mode fixture has all strict packs, so no validation error
        let checker = Checker {};
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/uses_strict_mode")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
            &1,
        )
        .unwrap();

        let error = checker.validate(&configuration);
        assert_eq!(error, None);
    }

    #[test]
    fn test_validate_structured_with_cycle() {
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/app_with_dependency_cycles")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
            &1,
        )
        .unwrap();

        let errors = validate_structured(&configuration);

        // Should have self_dependency and cycle errors
        let self_dep_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.error_type == "self_dependency")
            .collect();
        assert_eq!(self_dep_errors.len(), 1);
        assert!(self_dep_errors[0].message.contains("packs/baz/package.yml"));
        assert_eq!(
            self_dep_errors[0].file,
            Some("packs/baz/package.yml".to_string())
        );

        let cycle_errors: Vec<_> =
            errors.iter().filter(|e| e.error_type == "cycle").collect();
        assert_eq!(cycle_errors.len(), 1);
        let edges = cycle_errors[0].cycle_edges.as_ref().unwrap();
        assert_eq!(edges.len(), 2);
        // The cycle is foo -> bar -> foo (or bar -> foo -> bar)
        let pack_names: HashSet<String> =
            edges.iter().map(|e| e.from_pack.clone()).collect();
        assert!(pack_names.contains("packs/foo"));
        assert!(pack_names.contains("packs/bar"));
    }

    #[test]
    fn test_validate_structured_self_dependency() {
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/app_with_dependency_cycles")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
            &1,
        )
        .unwrap();

        let errors = validate_structured(&configuration);
        let self_dep: Vec<_> = errors
            .iter()
            .filter(|e| e.error_type == "self_dependency")
            .collect();
        assert_eq!(self_dep.len(), 1);
        assert_eq!(self_dep[0].file, Some("packs/baz/package.yml".to_string()));
        assert!(self_dep[0].cycle_edges.is_none());
    }

    #[test]
    fn test_validate_structured_no_errors() {
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/simple_app")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
            &1,
        )
        .unwrap();

        let errors = validate_structured(&configuration);
        assert!(errors.is_empty());
    }
}
