pub mod packs;

#[cfg(test)]
mod test_util {
    use configuration::Configuration;
    use packs::parsing::ruby::zeitwerk::get_zeitwerk_constant_resolver;
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    use packs::configuration;

    use crate::packs::configuration::from_raw;
    use crate::packs::constant_resolver::ConstantResolver;
    use crate::packs::pack::Pack;
    use crate::packs::raw_configuration::RawConfiguration;
    use crate::packs::walk_directory::WalkDirectoryResult;
    use crate::packs::{self};

    pub const SIMPLE_APP: &str = "tests/fixtures/simple_app";

    pub fn get_absolute_root(fixture_name: &str) -> PathBuf {
        PathBuf::from(fixture_name).canonicalize().unwrap()
    }

    pub fn get_zeitwerk_constant_resolver_for_fixture(
        fixture_name: &str,
    ) -> anyhow::Result<Box<dyn ConstantResolver>> {
        let absolute_root = get_absolute_root(fixture_name);
        let configuration = configuration::get(&absolute_root)?;

        Ok(get_zeitwerk_constant_resolver(
            &configuration.pack_set,
            &configuration.constant_resolver_configuration(),
        ))
    }

    // Note that instead, we could derive the `Default` trait on `Pack`
    // However, there should be no reason the "production" code ever initializes
    // a default Pack directly, so this implementation is test only.
    #[allow(clippy::derivable_impls)]
    impl Default for Pack {
        fn default() -> Self {
            Self {
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
                enforce_folder_privacy: Default::default(),
                enforce_folder_visibility: None,
                enforce_layers: Default::default(),
                client_keys: Default::default(),
                owner: Default::default(),
                enforcement_globs_ignore: Default::default(),
            }
        }
    }

    impl Default for Configuration {
        fn default() -> Self {
            let default_absolute_root = std::env::current_dir().unwrap();
            let root_pack = Pack {
                name: ".".to_owned(),
                ..Pack::default()
            };

            let included_packs: HashSet<Pack> =
                vec![root_pack].into_iter().collect();

            let walk_directory_result = WalkDirectoryResult {
                included_files: HashSet::new(),
                included_packs,
                owning_package_yml_for_file: HashMap::new(),
            };
            from_raw(
                &default_absolute_root,
                RawConfiguration::default(),
                walk_directory_result,
            )
            .unwrap() // TODO: potentially convert `default` to `new` and return a Result
        }
    }
}
