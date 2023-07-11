pub mod packs;

#[cfg(test)]
pub(crate) mod test_util {
    use configuration::Configuration;
    use packs::parsing::ruby::zeitwerk::get_zeitwerk_constant_resolver;
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    use packs::configuration;

    use crate::packs::configuration::from_raw;
    use crate::packs::pack::Pack;
    use crate::packs::raw_configuration::RawConfiguration;
    use crate::packs::walk_directory::WalkDirectoryResult;
    use crate::packs::{self, constant_resolver::ConstantResolver};

    pub const SIMPLE_APP: &str = "tests/fixtures/simple_app";

    pub fn get_absolute_root(fixture_name: &str) -> PathBuf {
        PathBuf::from(fixture_name).canonicalize().unwrap()
    }

    pub fn get_zeitwerk_constant_resolver_for_fixture(
        fixture_name: &str,
    ) -> Box<dyn ConstantResolver> {
        let absolute_root = get_absolute_root(fixture_name);
        let configuration = configuration::get(&absolute_root);

        get_zeitwerk_constant_resolver(
            &configuration.pack_set,
            &absolute_root,
            &configuration.cache_directory,
            true,
        )
    }

    // Note that instead, we could derive the `Default` trait on `Pack`
    // However, there should be no reason the "production" code ever initializes
    // a default Pack directly, so this implementation is test only.
    pub fn default_test_pack() -> Pack {
        Pack {
            yml: Default::default(),
            name: Default::default(),
            relative_path: Default::default(),
            dependencies: Default::default(),
            ignored_dependencies: Default::default(),
            ignored_private_constants: Default::default(),
            private_constants: Default::default(),
            package_todo: Default::default(),
            visible_to: Default::default(),
            public_folder: Default::default(),
            layer: Default::default(),
            enforce_dependencies: Default::default(),
            enforce_privacy: Default::default(),
            enforce_visibility: Default::default(),
            enforce_architecture: Default::default(),
        }
    }

    impl Default for Configuration {
        fn default() -> Self {
            let default_absolute_root = std::env::current_dir().unwrap();
            let walk_directory_result = WalkDirectoryResult {
                included_files: HashSet::new(),
                included_packs: HashSet::new(),
                owning_package_yml_for_file: HashMap::new(),
            };
            from_raw(
                &default_absolute_root,
                RawConfiguration::default(),
                walk_directory_result,
            )
        }
    }
}
