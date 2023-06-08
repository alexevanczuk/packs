pub struct ConstantResolver {}

impl ConstantResolver {
    fn create(
        configuration: Configuration,
        autoload_paths: Vec<String>,
    ) -> ConstantResolver {
        todo!()
    }
}

mod tests {
    use std::path::PathBuf;

    use crate::packs::Configuration;

    use super::*;

    #[test]
    fn test_create_from_load_paths() {
        let paths =
            vec!["app/services/".to_string(), "app/models/".to_string()];
        let absolute_root = PathBuf::from("tests/fixtures/simple_app")
            .canonicalize()
            .expect("Could not canonicalize path");
        let config = Configuration {
            absolute_root
            ..Configuration::default()
        };
        let resolver = ConstantResolver::create(config, paths);
        assert_eq!(
            resolver.get_pack_for(fully_qualified_constant).name,
            "packs/foo"
        )
    }
}
