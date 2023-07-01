use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use rayon::prelude::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::packs::{
    file_utils::process_glob_pattern,
    parsing::ruby::rails_utils::get_acronyms_from_disk, Pack, PackSet,
};

use super::packwerk::constant_resolver::{Constant, ConstantResolver};

#[derive(Serialize, Deserialize)]
struct ConstantResolverCache {
    file_definition_map: HashMap<PathBuf, String>,
}

fn inferred_constant_from_file(
    absolute_path: &Path,
    absolute_autoload_path: &PathBuf,
    acronyms: &HashSet<String>,
) -> Constant {
    let relative_path =
        absolute_path.strip_prefix(absolute_autoload_path).unwrap();

    let relative_path = relative_path.with_extension("");

    let relative_path_str = relative_path.to_str().unwrap();
    let fully_qualified_constant_name =
        crate::packs::inflector_shim::camelize(relative_path_str, acronyms);

    Constant {
        fully_qualified_name: fully_qualified_constant_name,
        absolute_path_of_definition: absolute_path.to_path_buf(),
    }
}

fn get_constant_resolver_cache(cache_dir: &Path) -> ConstantResolverCache {
    let path = cache_dir.join("constant_resolver.json");
    if path.exists() {
        let file = std::fs::File::open(path).unwrap();
        let reader = std::io::BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    } else {
        ConstantResolverCache {
            file_definition_map: HashMap::new(),
        }
    }
}

fn cache_constant_definitions(constants: &Vec<Constant>, cache_dir: &Path) {
    let mut file_definition_map: HashMap<PathBuf, String> = HashMap::new();
    for constant in constants {
        file_definition_map.insert(
            constant.absolute_path_of_definition.clone(),
            constant.fully_qualified_name.clone(),
        );
    }

    let cache_data_json = serde_json::to_string(&ConstantResolverCache {
        file_definition_map,
    })
    .expect("Failed to serialize");

    std::fs::create_dir_all(cache_dir).unwrap();
    std::fs::write(cache_dir.join("constant_resolver.json"), cache_data_json)
        .unwrap();
}

fn get_autoload_paths(packs: &Vec<Pack>) -> Vec<PathBuf> {
    let mut autoload_paths: Vec<PathBuf> = Vec::new();

    debug!(
        target: "perf_events",
        "Getting autoload paths"
    );

    for pack in packs {
        // App paths
        let app_paths = pack.yml.parent().unwrap().join("app").join("*");
        let app_glob_pattern = app_paths.to_str().unwrap();
        process_glob_pattern(app_glob_pattern, &mut autoload_paths);

        // Concerns paths
        let concerns_paths = pack
            .yml
            .parent()
            .unwrap()
            .join("app")
            .join("*")
            .join("concerns");
        let concerns_glob_pattern = concerns_paths.to_str().unwrap();

        process_glob_pattern(concerns_glob_pattern, &mut autoload_paths);
    }

    debug!(
        target: "perf_events",
        "Finished getting autoload paths"
    );

    autoload_paths
}

pub fn get_zeitwerk_constant_resolver(
    pack_set: &PackSet,
    absolute_root: &Path,
    cache_dir: &Path,
) -> ConstantResolver {
    let constants =
        inferred_constants_from_pack_set(pack_set, absolute_root, cache_dir);
    ConstantResolver::create(absolute_root, constants)
}

fn inferred_constants_from_pack_set(
    pack_set: &PackSet,
    absolute_root: &Path,
    cache_dir: &Path,
) -> Vec<Constant> {
    let autoload_paths = get_autoload_paths(&pack_set.packs);
    inferred_constants_from_autoload_paths(
        autoload_paths,
        absolute_root,
        cache_dir,
    )
}
fn inferred_constants_from_autoload_paths(
    autoload_paths: Vec<PathBuf>,
    absolute_root: &Path,
    cache_dir: &Path,
) -> Vec<Constant> {
    debug!(target: "perf_events", "Get constant resolver cache");
    let cache_data = get_constant_resolver_cache(cache_dir);

    debug!(target: "perf_events", "Globbing out autoload paths");
    // First, we get a map of each autoload path to the files they map to.
    let autoload_paths_to_their_globbed_files = autoload_paths
        .into_iter()
        .par_bridge()
        .map(|absolute_autoload_path| {
            let glob_path = absolute_autoload_path.join("**/*.rb");

            let files = glob::glob(glob_path.to_str().unwrap())
                .expect("Failed to read glob pattern")
                .filter_map(Result::ok)
                .collect::<Vec<PathBuf>>();

            (absolute_autoload_path, files)
        })
        .collect::<HashMap<PathBuf, Vec<PathBuf>>>();

    debug!(target: "perf_events", "Finding autoload path for each file");
    // Then, we want to know *which* autoload path is the one that defines a given constant.
    // The longest autoload path should be the one that does this.
    // For example, if we have two autoload paths:
    // 1) packs/my_pack/app/models
    // 2) packs/my_pack/app/models/concerns
    // And we have a file at `packs/my_pack/app/models/concerns/foo.rb`, we want to say that the constant `Foo` is defined by the second autoload path.
    // This is because the second autoload path is the longest path that contains the file.
    // We do this by creating a map of each file to the longest autoload path that contains it.
    let mut file_to_longest_path: HashMap<PathBuf, PathBuf> = HashMap::new();

    for (autoload_path, files) in &autoload_paths_to_their_globbed_files {
        for file in files {
            // Get the current longest path for this file, if it exists.
            let current_longest_path = file_to_longest_path
                .entry(file.clone())
                .or_insert_with(|| autoload_path.clone());

            // Update the longest path if the new path is longer.
            if autoload_path.components().count()
                > current_longest_path.components().count()
            {
                *current_longest_path = autoload_path.clone();
            }
        }
    }

    debug!(target: "perf_events", "Getting acronyms from disk");
    let acronyms = &get_acronyms_from_disk(absolute_root);

    debug!(target: "perf_events", "Inferring constants from file name (using cache)");
    let constants: Vec<Constant> = file_to_longest_path
        .into_iter()
        .par_bridge()
        .map(|(absolute_path_of_definition, absolute_autoload_path)| {
            if let Some(fully_qualified_name) = cache_data
                .file_definition_map
                .get(&absolute_path_of_definition)
            {
                Constant {
                    fully_qualified_name: fully_qualified_name.to_owned(),
                    absolute_path_of_definition,
                }
            } else {
                inferred_constant_from_file(
                    &absolute_path_of_definition,
                    &absolute_autoload_path,
                    acronyms,
                )
            }
        })
        .collect::<Vec<Constant>>();

    debug!(target: "perf_events", "Caching constant definitions");
    cache_constant_definitions(&constants, cache_dir);

    constants
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs::{
        configuration, parsing::ruby::packwerk::constant_resolver::Constant,
    };

    use pretty_assertions::assert_eq;

    #[test]
    fn test_file_map() {
        let absolute_root = &PathBuf::from("tests/fixtures/simple_app")
            .canonicalize()
            .expect("Could not canonicalize path");

        let configuration = configuration::get(absolute_root);

        let pack_set = configuration.pack_set;

        let constant_resolver = get_zeitwerk_constant_resolver(
            &pack_set,
            absolute_root,
            &configuration.cache_directory,
        );
        let actual_constant_map =
            constant_resolver.fully_qualified_constant_to_constant_map;

        let mut expected_constant_map = HashMap::new();
        expected_constant_map.insert(
            String::from("Foo::Bar"),
            Constant {
                fully_qualified_name: "Foo::Bar".to_owned(),
                absolute_path_of_definition: absolute_root
                    .join("packs/foo/app/services/foo/bar.rb"),
            },
        );

        expected_constant_map.insert(
            "Bar".to_owned(),
            Constant {
                fully_qualified_name: "Bar".to_owned(),
                absolute_path_of_definition: absolute_root
                    .join("packs/bar/app/services/bar.rb"),
            },
        );
        expected_constant_map.insert(
            "Baz".to_owned(),
            Constant {
                fully_qualified_name: "Baz".to_owned(),
                absolute_path_of_definition: absolute_root
                    .join("packs/baz/app/services/baz.rb"),
            },
        );
        expected_constant_map.insert(
            "Foo".to_owned(),
            Constant {
                fully_qualified_name: "Foo".to_owned(),
                absolute_path_of_definition: absolute_root
                    .join("packs/foo/app/services/foo.rb"),
            },
        );
        expected_constant_map.insert(
            "SomeConcern".to_owned(),
            Constant {
                fully_qualified_name: "SomeConcern".to_owned(),
                absolute_path_of_definition: absolute_root
                    .join("packs/bar/app/models/concerns/some_concern.rb"),
            },
        );
        expected_constant_map.insert(
            "SomeRootClass".to_owned(),
            Constant {
                fully_qualified_name: "SomeRootClass".to_owned(),
                absolute_path_of_definition: absolute_root
                    .join("app/services/some_root_class.rb"),
            },
        );
        assert_eq!(expected_constant_map, actual_constant_map);
    }
}
