use crate::packs::Pack;
use std::path::Path;

#[derive(Default)]
pub struct ConstantResolver {}

#[allow(unused_variables)]
impl ConstantResolver {
    pub fn create(
        absolute_root: &Path,
        autoload_paths: Vec<String>,
    ) -> ConstantResolver {
        todo!()
    }

    #[allow(dead_code)]
    fn get_pack_for(
        &self,
        constant: String,
        namespace_path: Vec<String>,
    ) -> Pack {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_create_from_load_paths() {
        let paths =
            vec!["app/services/".to_string(), "app/models/".to_string()];
        let absolute_root = PathBuf::from("tests/fixtures/simple_app")
            .canonicalize()
            .expect("Could not canonicalize path");

        let resolver = ConstantResolver::create(&absolute_root, paths);
        assert_eq!(
            resolver.get_pack_for(String::from("Foo"), vec![]).name,
            "packs/foo"
        )
    }
}
