use itertools::Itertools;

#[allow(unused_imports)]
use crate::packs::Pack;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[allow(dead_code)]
#[derive(Default)]
pub struct ConstantResolver {
    fully_qualified_constant_to_absolute_path_map: HashMap<String, PathBuf>,
    // Just for testing
    pub(crate) autoload_paths: Vec<PathBuf>,
}

#[derive(Debug, PartialEq)]
pub struct Constant {
    pub fully_qualified_name: String,
    pub absolute_path_of_definition: PathBuf,
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
        let mut fully_qualified_constant_to_absolute_path_map: HashMap<
            String,
            PathBuf,
        > = HashMap::new();

        for absolute_autoload_path in &autoload_paths {
            let mut glob_path = absolute_autoload_path.clone();
            glob_path.push("**/*.rb");

            let files = glob::glob(glob_path.to_str().unwrap())
                .expect("Failed to read glob pattern")
                .filter_map(Result::ok);

            for file in files {
                let relative_path =
                    file.strip_prefix(absolute_autoload_path).unwrap();

                let relative_path = relative_path.with_extension("");

                let fully_qualified_constant_name = relative_path
                    .to_str()
                    .unwrap()
                    .split('/')
                    .map(|s| s.to_string())
                    .map(|s| crate::packs::inflector_shim::to_class_case(&s))
                    .join("::");

                // Prefix each constant with :: to indicate it's an absolute reference
                let fully_qualified_constant_name =
                    format!("::{}", fully_qualified_constant_name);

                fully_qualified_constant_to_absolute_path_map
                    .insert(fully_qualified_constant_name, file);
            }
        }
        ConstantResolver {
            fully_qualified_constant_to_absolute_path_map,
            autoload_paths,
        }
    }

    #[allow(dead_code)]
    fn resolve(
        &self,
        fully_or_partially_qualified_constant: String,
        namespace_path: Vec<String>,
    ) -> Option<Constant> {
        // Example for namespace_path: ['Foo', 'Bar', 'Baz']
        // If the fully_or_partially_qualified_constant is 'Boo',
        // it could refer to any of the following:
        // ::Foo::Bar::Baz::Boo
        // ::Foo::Bar::Boo
        // ::Foo::Boo
        // ::Boo
        // We need to check each of these possibilities in order, and return the first one that exists
        // If none of them exist, return None

        // It's pretty common to refer to a name via a global reference, e.g. ::Foo::Bar::Baz::Boo
        // so we check that first
        let global_reference =
            format!("::{}", fully_or_partially_qualified_constant);
        if let Some(absolute_path) = self
            .fully_qualified_constant_to_absolute_path_map
            .get(&global_reference)
        {
            return Some(Constant {
                fully_qualified_name: global_reference,
                absolute_path_of_definition: absolute_path.clone(),
            });
        }

        let mut namespace_path = namespace_path;
        namespace_path.reverse();
        for _ in 0..namespace_path.len() {
            let candidate_namespace =
                namespace_path.clone().into_iter().rev().join("::");

            // Append the fully_or_partially_qualified_constant to the candidate_namespace
            let possible_constant = format!(
                "{}::{}",
                candidate_namespace, fully_or_partially_qualified_constant
            );

            if let Some(absolute_path) = self
                .fully_qualified_constant_to_absolute_path_map
                .get(&possible_constant)
            {
                return Some(Constant {
                    fully_qualified_name: possible_constant,
                    absolute_path_of_definition: absolute_path.clone(),
                });
            }
        }

        None
    }
}

#[allow(unused_imports)]
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

        let mut expected_file_map: HashMap<String, PathBuf> = HashMap::new();
        expected_file_map.insert(
            "::Foo".to_string(),
            PathBuf::from(
                "tests/fixtures/simple_app/packs/foo/app/services/foo.rb",
            ),
        );

        let actual_file_map =
            &resolver.fully_qualified_constant_to_absolute_path_map;

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
            resolver.resolve(String::from("Foo"), vec![]).unwrap()
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
                    String::from("Foo"),
                    vec![
                        String::from("Foo"),
                        String::from("Bar"),
                        String::from("Baz")
                    ]
                )
                .unwrap()
        )
    }
}
