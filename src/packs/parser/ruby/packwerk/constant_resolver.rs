use rayon::prelude::{ParallelBridge, ParallelIterator};
use regex::Regex;
use tracing::debug;

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct ConstantResolver {
    fully_qualified_constant_to_constant_map: HashMap<String, Constant>,
    // Just for testing
    #[allow(dead_code)]
    pub(crate) autoload_paths: Vec<PathBuf>,
}

#[derive(Debug, PartialEq)]
pub struct Constant {
    pub fully_qualified_name: String,
    pub absolute_path_of_definition: PathBuf,
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
#[allow(unused_variables)]
impl ConstantResolver {
    pub fn create(
        absolute_root: &Path,
        autoload_paths: Vec<PathBuf>,
    ) -> ConstantResolver {
        // For each autoload path, do the following:
        // 1) Glob for the ruby files
        // 2) For each ruby file, remove the autoloaded portion of the path
        // 3) For the remaining path, remove the .rb extension
        // 4) For the remaining path, split it by "/"
        // 5) Call packs::inflector::to_class_case on each element in the vector
        // 6) Join the vector with ::
        // 7) Strip the leading "::" from the string
        // 8) Add the fully qualified constant name to the map, with the value being the absolute path of the file
        let mut fully_qualified_constant_to_constant_map: HashMap<
            String,
            Constant,
        > = HashMap::new();

        debug!("Building constant resolver");

        let mut acronyms: HashSet<String> = HashSet::new();

        // Load in config/initializers/inflections.rb
        // For any inflections in there, add them to the acronyms vector
        // An inflection takes the form of "inflect.acronym 'API'", so "API" would be the acronym here
        // This is a bit of a hack, but it's the easiest way to get the inflections loaded in
        // TODO: Figure out a better way to do this
        let inflections_path =
            absolute_root.join("config/initializers/inflections.rb");
        if inflections_path.exists() {
            let inflections_file =
                std::fs::read_to_string(inflections_path).unwrap();
            let inflections_lines = inflections_file.lines();
            for line in inflections_lines {
                if line.contains(".acronym") {
                    let re = Regex::new(r#"['\\"]"#).unwrap();
                    let acronym = re.split(line).nth(1).unwrap();
                    acronyms.insert(acronym.to_string());
                }
            }
        }

        let constants: Vec<Constant> = autoload_paths
            .iter()
            .par_bridge()
            .flat_map(|absolute_autoload_path| {
                let mut glob_path = absolute_autoload_path.clone();
                glob_path.push("**/*.rb");

                let files = glob::glob(glob_path.to_str().unwrap())
                    .expect("Failed to read glob pattern")
                    .filter_map(Result::ok);

                files
                    .map(|file| {
                        inferred_constant_from_file(
                            &file,
                            absolute_autoload_path,
                            &acronyms,
                        )
                    })
                    .collect::<Vec<Constant>>()
            })
            .collect();

        for constant in constants {
            let fully_qualified_constant_name =
                constant.fully_qualified_name.clone();

            fully_qualified_constant_to_constant_map
                .insert(fully_qualified_constant_name, constant);
        }

        debug!("Finished building constant resolver");

        ConstantResolver {
            fully_qualified_constant_to_constant_map,
            autoload_paths,
        }
    }

    pub fn resolve(
        &self,
        fully_or_partially_qualified_constant: &str,
        namespace_path: &[String],
    ) -> Option<Constant> {
        // If the fully_or_partially_qualified_constant is prefixed with ::, the namespace path is technically empty, since it's a global reference
        let namespace_path =
            if fully_or_partially_qualified_constant.starts_with("::") {
                vec![]
            } else {
                namespace_path.to_vec()
            };

        let const_name = fully_or_partially_qualified_constant
            .trim_start_matches("::")
            .to_owned();

        self.resolve_constant(const_name.clone(), namespace_path, const_name)
    }

    fn resolve_constant(
        &self,
        const_name: String,
        current_namespace_path: Vec<String>,
        original_name: String,
    ) -> Option<Constant> {
        let constant = self.resolve_traversing_namespace_path(
            const_name.clone(),
            current_namespace_path.clone(),
        );
        match constant {
            (Some(namespace), Some(absolute_path_of_definition)) => {
                let mut fully_qualified_name_vec = vec![String::from("")];
                fully_qualified_name_vec.extend(namespace);
                fully_qualified_name_vec.push(original_name);
                let fully_qualified_name_guess =
                    fully_qualified_name_vec.join("::");

                Some(Constant {
                    fully_qualified_name: fully_qualified_name_guess,
                    absolute_path_of_definition,
                })
            }
            (None, None) => {
                // If we couldn't find a match, it's possible the constant is defined within its parent namespace and not within its own file.
                // For example, `Boo` above could be defined in `foo/bar.rb` as:
                // module Foo
                //   module Bar
                //     class Boo
                //     end
                //   end
                // end
                // Therefore, we take the given const_name, remove the last part of the fully qualified name, and try again.
                // In this case, we'd try to resolve `::Foo::Bar` instead of `::Foo::Bar::Boo`
                let split_const = const_name.split("::").collect::<Vec<&str>>();
                if split_const.len() <= 1 {
                    return None;
                }
                let parent_constant =
                    split_const[0..=split_const.len() - 2].join("::");
                self.resolve_constant(
                    parent_constant,
                    current_namespace_path,
                    original_name,
                )
            }
            _ => {
                todo!()
            }
        }
    }

    // Example for namespace_path: ['Foo', 'Bar', 'Baz']
    // If the const_name is 'Boo',
    // it could refer to any of the following:
    // ::Foo::Bar::Baz::Boo
    // ::Foo::Bar::Boo
    // ::Foo::Boo
    // ::Boo
    // We need to check each of these possibilities in order, and return the first one that exists
    // If none of them exist, return None
    fn resolve_traversing_namespace_path(
        &self,
        const_name: String,
        current_namespace_path: Vec<String>,
    ) -> (Option<Vec<String>>, Option<PathBuf>) {
        let mut fully_qualified_name_guess_vec = current_namespace_path.clone();
        fully_qualified_name_guess_vec.push(const_name.clone());
        let fully_qualified_name_guess =
            fully_qualified_name_guess_vec.join("::");
        // Join current_namespace_path and const_name with "::"
        // current_namespace_path
        //     .iter()
        //     .chain(std::iter::once(const_name))
        //     .join("::");
        if let Some(constant) =
            self.constant_for_fully_qualified_name(&fully_qualified_name_guess)
        {
            let x = current_namespace_path;
            let y = constant.absolute_path_of_definition.clone();
            (Some(x), Some(y))
        } else {
            // In this case, we couldn't find a constant with the given name under the given namespace.
            // However, it's possible the constant is defined within the parent namespace.
            let split_result = current_namespace_path.split_last();
            match split_result {
                Some((_last, parent_namespace)) => {
                    let (namespace, absolute_path_of_definition) = self
                        .resolve_traversing_namespace_path(
                            const_name,
                            parent_namespace.to_vec(),
                        );
                    (namespace, absolute_path_of_definition)
                }
                None => (None, None),
            }
        }
    }

    fn constant_for_fully_qualified_name(
        &self,
        fully_qualified_name: &String,
    ) -> Option<&Constant> {
        if let Some(constant) = self
            .fully_qualified_constant_to_constant_map
            .get(fully_qualified_name)
        {
            return Some(constant);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::packs::configuration;

    use super::*;

    #[test]
    fn test_file_map() {
        let paths = vec![PathBuf::from(
            "tests/fixtures/simple_app/packs/foo/app/services",
        )];
        let absolute_root = PathBuf::from("tests/fixtures/simple_app")
            .canonicalize()
            .expect("Could not canonicalize path");

        let resolver = ConstantResolver::create(&absolute_root, paths);

        let mut expected_file_map: HashMap<String, Constant> = HashMap::new();
        expected_file_map.insert(
            "Foo".to_string(),
            Constant {
                fully_qualified_name: "Foo".to_string(),
                absolute_path_of_definition: PathBuf::from(
                    "tests/fixtures/simple_app/packs/foo/app/services/foo.rb",
                ),
            },
        );

        expected_file_map.insert(
            "Foo::Bar".to_string(),
            Constant {
                fully_qualified_name: "Foo::Bar".to_string(),
                absolute_path_of_definition: PathBuf::from(
                    "tests/fixtures/simple_app/packs/foo/app/services/foo/bar.rb",
                ),
            }
        );

        let actual_file_map =
            &resolver.fully_qualified_constant_to_constant_map;

        assert_eq!(&expected_file_map, actual_file_map);
    }

    #[test]
    fn unnested_reference_to_unnested_constant() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app")
            .canonicalize()
            .unwrap();
        let resolver = configuration::get(&absolute_root).constant_resolver;

        assert_eq!(
            Constant {
                fully_qualified_name: "::Foo".to_string(),
                absolute_path_of_definition: absolute_root
                    .join("packs/foo/app/services/foo.rb")
            },
            resolver.resolve(&String::from("Foo"), &[]).unwrap()
        )
    }
    #[test]
    fn nested_reference_to_unnested_constant() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app")
            .canonicalize()
            .unwrap();
        let resolver = configuration::get(&absolute_root).constant_resolver;
        assert_eq!(
            Constant {
                fully_qualified_name: "::Foo".to_string(),
                absolute_path_of_definition: absolute_root
                    .join("packs/foo/app/services/foo.rb")
            },
            resolver
                .resolve(
                    &String::from("Foo"),
                    &[
                        String::from("Foo"),
                        String::from("Bar"),
                        String::from("Baz")
                    ]
                )
                .unwrap()
        )
    }

    #[test]
    fn nested_reference_to_nested_constant() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app")
            .canonicalize()
            .unwrap();
        let resolver = configuration::get(&absolute_root).constant_resolver;
        assert_eq!(
            Constant {
                fully_qualified_name: "::Foo::Bar".to_string(),
                absolute_path_of_definition: absolute_root
                    .join("packs/foo/app/services/foo/bar.rb")
            },
            resolver
                .resolve(&String::from("Bar"), &[String::from("Foo")])
                .unwrap()
        )
    }

    #[test]
    fn nested_reference_to_global_constant() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app")
            .canonicalize()
            .unwrap();
        let resolver = configuration::get(&absolute_root).constant_resolver;
        assert_eq!(
            Constant {
                fully_qualified_name: "::Bar".to_string(),
                absolute_path_of_definition: absolute_root
                    .join("packs/bar/app/services/bar.rb")
            },
            resolver
                .resolve(&String::from("::Bar"), &[String::from("Foo")])
                .unwrap()
        )
    }

    #[test]
    fn nested_reference_to_constant_defined_within_another_file() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app")
            .canonicalize()
            .unwrap();
        let resolver = configuration::get(&absolute_root).constant_resolver;
        assert_eq!(
            Constant {
                fully_qualified_name: "::Bar::BAR".to_string(),
                absolute_path_of_definition: absolute_root
                    .join("packs/bar/app/services/bar.rb")
            },
            resolver.resolve(&String::from("::Bar::BAR"), &[]).unwrap()
        )
    }

    #[test]
    fn inflected_constant() {
        let absolute_root =
            PathBuf::from("tests/fixtures/app_with_inflections")
                .canonicalize()
                .unwrap();
        let resolver = configuration::get(&absolute_root).constant_resolver;

        assert_eq!(
            Constant {
                fully_qualified_name: "::MyModule::SomeAPIClass".to_string(),
                absolute_path_of_definition: absolute_root
                    .join("app/services/my_module/some_api_class.rb")
            },
            resolver
                .resolve(&String::from("::MyModule::SomeAPIClass"), &[])
                .unwrap()
        );

        assert_eq!(
            Constant {
                fully_qualified_name: "::MyModule::SomeCSVClass".to_string(),
                absolute_path_of_definition: absolute_root
                    .join("app/services/my_module/some_csv_class.rb")
            },
            resolver
                .resolve(&String::from("::MyModule::SomeCSVClass"), &[])
                .unwrap()
        )
    }
}
