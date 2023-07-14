use std::collections::HashMap;

use petgraph::prelude::DiGraph;

use super::{pack::Pack, Configuration};

pub(crate) fn of_dependencies(
    configuration: &Configuration,
) -> (
    DiGraph<(), ()>,
    HashMap<petgraph::prelude::NodeIndex, &Pack>,
) {
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
            let to_pack = configuration
                    .pack_set
                    .for_pack(dependency_pack_name)
                    .unwrap_or_else(|_| panic!("{} has '{}' in its dependencies, but that pack cannot be found. Try `packs list-packs` to debug.",
                        &pack.yml.to_string_lossy(),
                        dependency_pack_name));
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

    (graph, node_to_pack)
}

pub(crate) fn of_violations(
    configuration: &Configuration,
) -> (
    DiGraph<(), ()>,
    HashMap<petgraph::prelude::NodeIndex, &Pack>,
) {
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
        for violation in &pack.all_violations() {
            let from_pack = pack;
            let to_pack = configuration
                    .pack_set
                    .for_pack(&violation.defining_pack_name)
                    .unwrap_or_else(|_| panic!("{} has '{}' in its dependencies, but that pack cannot be found. Try `packs list-packs` to debug.",
                        &pack.yml.to_string_lossy(),
                        &violation.defining_pack_name));
            let from_node = pack_to_node
                .get(from_pack)
                .expect("Could not find from_pack")
                .to_owned();
            let to_node = pack_to_node
                .get(to_pack)
                .expect("Could not find to_pack")
                .to_owned();
            graph.add_edge(from_node, to_node, ());
        }
    }

    (graph, node_to_pack)
}
