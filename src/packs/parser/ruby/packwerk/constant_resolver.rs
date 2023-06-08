#[allow(unused_imports)]
use crate::packs::Pack;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[allow(dead_code)]
#[derive(Default)]
pub struct ConstantResolver {
    file_map: HashMap<String, String>,
    // Just for testing
    pub(crate) autoload_paths: Vec<String>,
}

pub struct Constant {
    pub fully_qualified_name: String,
    pub absolute_path_of_definition: PathBuf,
}
#[allow(unused_variables)]
impl ConstantResolver {
    pub fn create(
        absolute_root: &Path,
        autoload_paths: Vec<String>,
    ) -> ConstantResolver {
        ConstantResolver {
            file_map: HashMap::new(),
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

    // #[test]
    // fn trivial() {
    //     let paths =
    //         vec!["app/services/".to_string(), "app/models/".to_string()];
    //     let absolute_root = PathBuf::from("tests/fixtures/simple_app")
    //         .canonicalize()
    //         .expect("Could not canonicalize path");

    //     let resolver = ConstantResolver::create(&absolute_root, paths);
    //     assert_eq!(
    //         resolver.resolve(String::from("Foo"), vec![]),
    //         PathBuf::from(
    //             "tests/fixtures/simple_app/packs/foo/app/services/foo.rb"
    //         )
    //     )
    // }
}
