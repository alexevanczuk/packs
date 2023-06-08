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
        // 5) Call inflector::cases::classcase::to_class_case
        // 5)
        ConstantResolver {
            fully_qualified_constant_to_absolute_path_map: HashMap::new(),
            autoload_paths,
        }
    }

    #[allow(dead_code)]
    fn resolve(
        &self,
        constant: String,
        namespace_path: Vec<String>,
    ) -> Constant {
        // TODO!
        Constant {
            fully_qualified_name: String::from(""),
            absolute_path_of_definition: PathBuf::from(""),
        }
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn trivial() {
        let paths =
            vec![PathBuf::from("tests/fixtures/simple_app/app/services")];
        let absolute_root = PathBuf::from("tests/fixtures/simple_app")
            .canonicalize()
            .expect("Could not canonicalize path");

        let resolver = ConstantResolver::create(&absolute_root, paths);

        let mut expected_file_map: HashMap<String, PathBuf> = HashMap::new();
        expected_file_map.insert(
            "Foo".to_string(),
            PathBuf::from("tests/fixtures/simple_app/app/services"),
        );

        let actual_file_map =
            resolver.fully_qualified_constant_to_absolute_path_map;

        assert_eq!(expected_file_map, actual_file_map);
        // assert_eq!(
        //     resolver.resolve(String::from("Foo"), vec![]),
        //     PathBuf::from(
        //         "tests/fixtures/simple_app/packs/foo/app/services/foo.rb"
        //     )
        // )
    }
}
