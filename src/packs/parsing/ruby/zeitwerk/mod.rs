mod constant_resolver;

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use rayon::prelude::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::packs::{
    caching::create_cache_dir_idempotently,
    constant_resolver::{
        ConstantDefinition, ConstantResolver, ConstantResolverConfiguration,
    },
    file_utils::expand_glob,
    pack::Pack,
    parsing::ruby::rails_utils::get_acronyms_from_disk,
    PackSet,
};

use self::constant_resolver::ZeitwerkConstantResolver;

use super::inflector_shim;

pub fn get_zeitwerk_constant_resolver(
    pack_set: &PackSet,
    configuration: &ConstantResolverConfiguration,
) -> Box<dyn ConstantResolver + Send + Sync> {
    let constants = inferred_constants_from_pack_set(pack_set, configuration);

    ZeitwerkConstantResolver::create(constants)
}

#[derive(Debug)]
struct PackNamespaceSettings {
    automatic_pack_namespace: bool,
    automatic_pack_namespace_exclusions: HashSet<PathBuf>,
}

fn get_pack_namespace_settings(pack: &Pack) -> PackNamespaceSettings {
    pack.client_keys
        .get("metadata")
        .and_then(|metadata| {
            if let serde_yaml::Value::Mapping(map) = metadata {
                // Extract automatic_pack_namespace
                let automatic_pack_namespace = map
                    .get(serde_yaml::Value::String(
                        "automatic_pack_namespace".to_string(),
                    ))
                    .and_then(|val| match val {
                        serde_yaml::Value::Bool(b) => Some(*b),
                        _ => None,
                    })
                    .unwrap_or(false); // Default to false if not found or not a boolean

                // Extract automatic_pack_namespace_exclusions and combine with pack.yml
                let automatic_pack_namespace_exclusions: HashSet<PathBuf> = map
                    .get(serde_yaml::Value::String(
                        "automatic_pack_namespace_exclusions".to_string(),
                    ))
                    .and_then(|val| match val {
                        serde_yaml::Value::Sequence(seq) => Some(
                            seq.iter()
                                .filter_map(|v| {
                                    v.as_str().map(|s| {
                                        // Combine pack.yml with the exclusion path to form the full absolute path
                                        let mut full_path = pack.yml.clone();
                                        full_path.pop(); // Remove the last component (usually the filename like "pack.yml")
                                        full_path.push(s); // Add the exclusion path
                                        full_path
                                    })
                                })
                                .collect(),
                        ),
                        _ => None,
                    })
                    .unwrap_or_default(); // Default to empty set if not found or not a sequence

                Some(PackNamespaceSettings {
                    automatic_pack_namespace,
                    automatic_pack_namespace_exclusions,
                })
            } else {
                None
            }
        })
        .unwrap_or(PackNamespaceSettings {
            automatic_pack_namespace: false,
            automatic_pack_namespace_exclusions: HashSet::new(),
        }) // Default to false and empty set if metadata doesn't exist
}

fn inferred_constants_from_pack_set(
    pack_set: &PackSet,
    configuration: &ConstantResolverConfiguration,
) -> Vec<ConstantDefinition> {
    // build the full list of default autoload roots from the pack set, using the default namespace for each.
    // There is one exception to using the default namespace:
    // Each pack may have metadata that takes this shape:
    // metadata:
    // automatic_pack_namespace: true
    // automatic_pack_namespace_exclusions:
    //     - app/models # Exclude models
    // For packs that have this configuration, if the autoload root is not in the list of automatic_pack_namespace_exclusions,
    // set the namespace associated with that root to inflector_shim::camelize(pack.name).
    let mut full_autoload_roots: HashMap<PathBuf, String> = pack_set
        .packs
        .iter()
        .flat_map(|pack| {
            let default_roots = pack.default_autoload_roots();

            // Check if metadata exists and automatic_pack_namespace is set to true

            let PackNamespaceSettings {
                automatic_pack_namespace,
                automatic_pack_namespace_exclusions,
            } = get_pack_namespace_settings(pack);

            // Build the autoload roots
            default_roots.into_iter().map(move |path| {
                if automatic_pack_namespace
                    && !automatic_pack_namespace_exclusions.contains(&path)
                {
                    // Pass an empty set of acronyms as the second argument
                    // NOTE: This is not the correct implementation – if we want automatic namespacing to work with
                    // acronym-based pack names, we need to pull from the file, preferably from the cache.
                    let empty_acronyms = HashSet::new();

                    // Camelized pack namespace based on pack name with leading double colon:
                    // e.g. pack name "packs/my_pack" -> "::MyPack"
                    let namespace = format!(
                        "::{}",
                        inflector_shim::camelize(
                            pack.last_name(),
                            &empty_acronyms,
                        )
                    );

                    (path, namespace)
                } else {
                    (path, String::from("")) // default namespace handling
                }
            })
        })
        .collect();

    // override the default autoload roots with any that may have been explicitly specified.
    configuration
        .autoload_roots
        .iter()
        .for_each(|(rel_path, ns)| {
            let abs_path = configuration.absolute_root.join(rel_path);
            let ns = if ns == "::Object" {
                String::from("")
            } else {
                ns.to_owned()
            };
            expand_glob(abs_path.to_str().unwrap())
                .iter()
                .for_each(|path| {
                    full_autoload_roots.insert(path.to_owned(), ns.clone());
                });
        });

    inferred_constants_from_autoload_paths(configuration, full_autoload_roots)
}

fn inferred_constants_from_autoload_paths(
    configuration: &ConstantResolverConfiguration,
    full_autoload_roots: HashMap<PathBuf, String>,
) -> Vec<ConstantDefinition> {
    debug!("Get constant resolver cache");
    let cache_data = get_constant_resolver_cache(configuration.cache_directory);

    debug!("Globbing out autoload paths");
    // First, we get a map of each autoload path to the files they map to.
    let autoload_paths_to_their_globbed_files = full_autoload_roots
        .keys()
        .par_bridge()
        .map(|absolute_autoload_path| {
            let glob_path = absolute_autoload_path.join("**/*.rb");

            let files = glob::glob(glob_path.to_str().unwrap())
                .expect("Failed to read glob pattern")
                .filter_map(Result::ok)
                .collect::<Vec<PathBuf>>();

            (absolute_autoload_path, files)
        })
        .collect::<HashMap<&PathBuf, Vec<PathBuf>>>();

    debug!("Finding autoload path for each file");
    // Then, we want to know *which* autoload path is the one that defines a given constant.
    // The longest autoload path should be the one that does this.
    // For example, if we have two autoload paths:
    // 1) packs/my_pack/app/models
    // 2) packs/my_pack/app/models/concerns
    // And we have a file at `packs/my_pack/app/models/concerns/foo.rb`, we want to say that the constant `Foo` is defined by the second autoload path.
    // This is because the second autoload path is the longest path that contains the file.
    // We do this by creating a map of each file to the longest autoload path that contains it.
    let mut file_to_longest_path: HashMap<&PathBuf, &PathBuf> = HashMap::new();

    for (autoload_path, files) in &autoload_paths_to_their_globbed_files {
        for file in files {
            // Get the current longest path for this file, if it exists.
            let current_longest_path = file_to_longest_path
                .entry(file)
                .or_insert_with(|| autoload_path);

            // Update the longest path if the new path is longer.
            if autoload_path.components().count()
                > current_longest_path.components().count()
            {
                *current_longest_path = autoload_path;
            }
        }
    }

    debug!("Getting acronyms from disk");
    let acronyms = &get_acronyms_from_disk(configuration.inflections_path);

    debug!("Inferring constants from file name (using cache)");
    let constants: Vec<ConstantDefinition> = file_to_longest_path
        .into_iter()
        .par_bridge()
        .map(|(absolute_path_of_definition, absolute_autoload_path)| {
            if let Some(fully_qualified_name) = cache_data
                .file_definition_map
                .get(absolute_path_of_definition)
            {
                ConstantDefinition {
                    fully_qualified_name: fully_qualified_name.to_owned(),
                    absolute_path_of_definition: absolute_path_of_definition
                        .to_owned(),
                }
            } else {
                let default_namespace =
                    full_autoload_roots.get(absolute_autoload_path).unwrap();
                inferred_constant_from_file(
                    absolute_path_of_definition,
                    absolute_autoload_path,
                    acronyms,
                    default_namespace,
                )
            }
        })
        .collect::<Vec<ConstantDefinition>>();

    debug!("Caching constant definitions");
    cache_constant_definitions(
        &constants,
        configuration.cache_directory,
        !configuration.cache_enabled,
    );

    constants
}

fn inferred_constant_from_file(
    absolute_path: &Path,
    absolute_autoload_path: &PathBuf,
    acronyms: &HashSet<String>,
    default_namespace: &String,
) -> ConstantDefinition {
    let relative_path =
        absolute_path.strip_prefix(absolute_autoload_path).unwrap();

    let relative_path = relative_path.with_extension("");

    let relative_path_str = relative_path.to_str().unwrap();
    let camelized_path = inflector_shim::camelize(relative_path_str, acronyms);
    let fully_qualified_name =
        format!("{}::{}", default_namespace, camelized_path);

    ConstantDefinition {
        fully_qualified_name,
        absolute_path_of_definition: absolute_path.to_path_buf(),
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct ConstantResolverCache {
    file_definition_map: HashMap<PathBuf, String>,
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

fn cache_constant_definitions(
    constants: &Vec<ConstantDefinition>,
    cache_dir: &Path,
    cache_disabled: bool,
) {
    if cache_disabled {
        return;
    }

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

    create_cache_dir_idempotently(cache_dir);
    std::fs::write(cache_dir.join("constant_resolver.json"), cache_data_json)
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs;
    use crate::packs::configuration;

    fn teardown() {
        packs::delete_cache(
            configuration::get(&PathBuf::from("tests/fixtures/simple_app"), &0)
                .unwrap(),
        );
    }

    use crate::test_util::{
        get_absolute_root, get_zeitwerk_constant_resolver_for_fixture,
        SIMPLE_APP,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn unnested_reference_to_unnested_constant() {
        assert_eq!(
            vec![ConstantDefinition {
                fully_qualified_name: "::Foo".to_string(),
                absolute_path_of_definition: get_absolute_root(SIMPLE_APP)
                    .join("packs/foo/app/services/foo.rb")
            }],
            get_zeitwerk_constant_resolver_for_fixture(SIMPLE_APP)
                .unwrap()
                .resolve(&String::from("Foo"), &[])
                .unwrap()
        );

        teardown();
    }

    #[test]
    fn constant_in_overridden_namespace() {
        assert_eq!(
            vec![ConstantDefinition {
                fully_qualified_name: "::Company::Widget".to_string(),
                absolute_path_of_definition: get_absolute_root(SIMPLE_APP)
                    .join("app/company_data/widget.rb")
            }],
            get_zeitwerk_constant_resolver_for_fixture(SIMPLE_APP)
                .unwrap()
                .resolve(&String::from("Widget"), &["Company"])
                .unwrap()
        );

        teardown();
    }

    #[test]
    fn nested_reference_to_unnested_constant() {
        let absolute_root = get_absolute_root(SIMPLE_APP);
        let resolver =
            get_zeitwerk_constant_resolver_for_fixture(SIMPLE_APP).unwrap();

        assert_eq!(
            vec![ConstantDefinition {
                fully_qualified_name: "::Foo".to_string(),
                absolute_path_of_definition: absolute_root
                    .join("packs/foo/app/services/foo.rb")
            }],
            resolver
                .resolve(&String::from("Foo"), &["Foo", "Bar", "Baz"])
                .unwrap()
        );

        teardown();
    }

    #[test]
    fn nested_reference_to_nested_constant() {
        let absolute_root = get_absolute_root(SIMPLE_APP);
        let resolver =
            get_zeitwerk_constant_resolver_for_fixture(SIMPLE_APP).unwrap();
        assert_eq!(
            vec![ConstantDefinition {
                fully_qualified_name: "::Foo::Bar".to_string(),
                absolute_path_of_definition: absolute_root
                    .join("packs/foo/app/services/foo/bar.rb")
            }],
            resolver.resolve("Bar", &["Foo"]).unwrap()
        );

        teardown();
    }

    #[test]
    fn nested_reference_to_global_constant() {
        let absolute_root = get_absolute_root(SIMPLE_APP);
        let resolver =
            get_zeitwerk_constant_resolver_for_fixture(SIMPLE_APP).unwrap();

        assert_eq!(
            vec![ConstantDefinition {
                fully_qualified_name: "::Bar".to_string(),
                absolute_path_of_definition: absolute_root
                    .join("packs/bar/app/services/bar.rb")
            }],
            resolver.resolve("::Bar", &["Foo"]).unwrap()
        );

        teardown();
    }

    #[test]
    fn nested_reference_to_constant_defined_within_another_file() {
        let absolute_root = get_absolute_root(SIMPLE_APP);
        let resolver =
            get_zeitwerk_constant_resolver_for_fixture(SIMPLE_APP).unwrap();
        assert_eq!(
            vec![ConstantDefinition {
                fully_qualified_name: "::Bar::BAR".to_string(),
                absolute_path_of_definition: absolute_root
                    .join("packs/bar/app/services/bar.rb")
            }],
            resolver.resolve(&String::from("::Bar::BAR"), &[]).unwrap()
        );

        teardown();
    }

    #[test]
    fn inflected_constant() {
        let app = "tests/fixtures/app_with_inflections";
        let absolute_root = get_absolute_root(app);
        let resolver = get_zeitwerk_constant_resolver_for_fixture(app).unwrap();

        assert_eq!(
            vec![ConstantDefinition {
                fully_qualified_name: "::MyModule::SomeAPIClass".to_string(),
                absolute_path_of_definition: absolute_root
                    .join("app/services/my_module/some_api_class.rb")
            }],
            resolver
                .resolve(&String::from("::MyModule::SomeAPIClass"), &[])
                .unwrap()
        );

        assert_eq!(
            vec![ConstantDefinition {
                fully_qualified_name: "::MyModule::SomeCSVClass".to_string(),
                absolute_path_of_definition: absolute_root
                    .join("app/services/my_module/some_csv_class.rb")
            }],
            resolver
                .resolve(&String::from("::MyModule::SomeCSVClass"), &[])
                .unwrap()
        );

        teardown();
    }

    #[test]
    fn test_file_map() {
        let absolute_root = &PathBuf::from("tests/fixtures/simple_app")
            .canonicalize()
            .expect("Could not canonicalize path");

        let configuration = configuration::get(absolute_root, &0).unwrap();

        let constant_resolver = get_zeitwerk_constant_resolver(
            &configuration.pack_set,
            &configuration.constant_resolver_configuration(),
        );
        let actual_constant_map = constant_resolver
            .fully_qualified_constant_name_to_constant_definition_map();

        let mut expected_constant_map = HashMap::new();
        expected_constant_map.insert(
            String::from("::Foo::Bar"),
            vec![ConstantDefinition {
                fully_qualified_name: "::Foo::Bar".to_owned(),
                absolute_path_of_definition: absolute_root
                    .join("packs/foo/app/services/foo/bar.rb"),
            }],
        );

        expected_constant_map.insert(
            "::Bar".to_owned(),
            vec![ConstantDefinition {
                fully_qualified_name: "::Bar".to_owned(),
                absolute_path_of_definition: absolute_root
                    .join("packs/bar/app/services/bar.rb"),
            }],
        );
        expected_constant_map.insert(
            "::Baz".to_owned(),
            vec![ConstantDefinition {
                fully_qualified_name: "::Baz".to_owned(),
                absolute_path_of_definition: absolute_root
                    .join("packs/baz/app/services/baz.rb"),
            }],
        );
        expected_constant_map.insert(
            "::Foo".to_owned(),
            vec![ConstantDefinition {
                fully_qualified_name: "::Foo".to_owned(),
                absolute_path_of_definition: absolute_root
                    .join("packs/foo/app/services/foo.rb"),
            }],
        );
        expected_constant_map.insert(
            "::SomeConcern".to_owned(),
            vec![ConstantDefinition {
                fully_qualified_name: "::SomeConcern".to_owned(),
                absolute_path_of_definition: absolute_root
                    .join("packs/bar/app/models/concerns/some_concern.rb"),
            }],
        );
        expected_constant_map.insert(
            "::SomeRootClass".to_owned(),
            vec![ConstantDefinition {
                fully_qualified_name: "::SomeRootClass".to_owned(),
                absolute_path_of_definition: absolute_root
                    .join("app/services/some_root_class.rb"),
            }],
        );
        expected_constant_map.insert(
            "::Company::Widget".to_owned(),
            vec![ConstantDefinition {
                fully_qualified_name: "::Company::Widget".to_owned(),
                absolute_path_of_definition: absolute_root
                    .join("app/company_data/widget.rb"),
            }],
        );

        assert_eq!(&expected_constant_map, actual_constant_map);
        teardown();
    }

    #[test]
    fn test_cache_constant_definitions() {
        let absolute_root = &PathBuf::from("tests/fixtures/simple_app")
            .canonicalize()
            .expect("Could not canonicalize path");

        let configuration = configuration::get(absolute_root, &0).unwrap();

        let constant_resolver = get_zeitwerk_constant_resolver(
            &configuration.pack_set,
            &configuration.constant_resolver_configuration(),
        );
        let constants = constant_resolver
            .fully_qualified_constant_name_to_constant_definition_map();

        let cache_dir = configuration
            .constant_resolver_configuration()
            .cache_directory
            .clone();
        cache_constant_definitions(
            &constants.values().flatten().cloned().collect(),
            &cache_dir,
            false,
        );

        let cache_data = get_constant_resolver_cache(&cache_dir);

        // ~/workspace/packs - main ! $ tree tests/fixtures/simple_app
        // tests/fixtures/simple_app
        // ├── app
        // │   ├── company_data
        // │   │   └── widget.rb
        // │   └── services
        // │       └── some_root_class.rb
        // ├── frontend
        // │   └── ui_helper.rb
        // ├── node_modules
        // │   ├── file.rb
        // │   └── subfolder
        // │       └── file.rb
        // ├── package.yml
        // ├── packs
        // │   ├── bar
        // │   │   ├── app
        // │   │   │   ├── models
        // │   │   │   │   └── concerns
        // │   │   │   │       └── some_concern.rb
        // │   │   │   └── services
        // │   │   │       └── bar.rb
        // │   │   └── package.yml
        // │   ├── baz
        // │   │   ├── app
        // │   │   │   └── services
        // │   │   │       └── baz.rb
        // │   │   └── package.yml
        // │   └── foo
        // │       ├── app
        // │       │   ├── services
        // │       │   │   ├── foo
        // │       │   │   │   └── bar.rb
        // │       │   │   └── foo.rb
        // │       │   └── views
        // │       │       └── foo.erb
        // │       └── package.yml
        // ├── packwerk.yml
        // ├── script
        // │   └── my_script.rb
        // └── tmp
        let mut expected_file_definition_map = HashMap::new();

        expected_file_definition_map.insert(
            absolute_root.join("packs/foo/app/services/foo.rb"),
            "::Foo".to_string(),
        );

        expected_file_definition_map.insert(
            absolute_root.join("packs/foo/app/services/foo/bar.rb"),
            "::Foo::Bar".to_string(),
        );

        expected_file_definition_map.insert(
            absolute_root.join("packs/bar/app/services/bar.rb"),
            "::Bar".to_string(),
        );

        expected_file_definition_map.insert(
            absolute_root.join("packs/bar/app/models/concerns/some_concern.rb"),
            "::SomeConcern".to_string(),
        );

        expected_file_definition_map.insert(
            absolute_root.join("packs/baz/app/services/baz.rb"),
            "::Baz".to_string(),
        );

        expected_file_definition_map.insert(
            absolute_root.join("app/services/some_root_class.rb"),
            "::SomeRootClass".to_string(),
        );

        expected_file_definition_map.insert(
            absolute_root.join("app/company_data/widget.rb"),
            "::Company::Widget".to_string(),
        );

        assert_eq!(
            ConstantResolverCache {
                file_definition_map: expected_file_definition_map
            },
            cache_data
        );

        teardown();
    }

    use std::collections::HashMap;
    use std::path::PathBuf;
}
