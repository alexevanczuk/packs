use super::caching::InitializedCacheDirectory;
use super::file_utils::user_inputted_paths_to_absolute_filepaths;
use super::{checker::architecture::Layers, per_file_cache};
use super::{noop_cache, PackSet};
use crate::packs::caching::Cache;

use crate::packs::raw_configuration;
use crate::packs::walk_directory::WalkDirectoryResult;

use crate::packs::walk_directory;

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use tracing::debug;
use walk_directory::walk_directory;

#[derive(Default)]
pub struct Configuration {
    pub included_files: HashSet<PathBuf>,
    pub absolute_root: PathBuf,
    pub cache_enabled: bool,
    pub cache_directory: PathBuf,
    pub pack_set: PackSet,
    pub layers: Layers,
    pub experimental_parser: bool,
}

impl Configuration {
    pub(crate) fn intersect_files(
        &self,
        input_files: Vec<String>,
    ) -> HashSet<PathBuf> {
        if input_files.is_empty() {
            self.included_files.clone()
        } else {
            let absolute_filepaths = user_inputted_paths_to_absolute_filepaths(
                &self.absolute_root,
                input_files,
            );
            self.included_files
                .intersection(&absolute_filepaths)
                .cloned()
                .collect::<HashSet<PathBuf>>()
        }
    }

    pub(crate) fn get_cache(
        &self,
        // This takes an unused "InitializedCacheDirectory" argument, which is a sentinel
        // value returned by
        _initialized_dir: InitializedCacheDirectory,
    ) -> Box<dyn Cache + Send + Sync> {
        if self.cache_enabled {
            Box::new(per_file_cache::PerFileCache {
                cache_dir: self.cache_directory.to_owned(),
            })
        } else {
            Box::new(noop_cache::NoopCache {})
        }
    }

    #[allow(dead_code)]
    pub(crate) fn with_experimental_parser(self) -> Configuration {
        Configuration {
            experimental_parser: true,
            ..self
        }
    }
}

pub(crate) fn get(absolute_root: &Path) -> Configuration {
    debug!("Beginning to build configuration");

    let raw_config = raw_configuration::get(absolute_root);

    let WalkDirectoryResult {
        included_files,
        included_packs,
        owning_package_yml_for_file,
    } = walk_directory(absolute_root.to_path_buf(), &raw_config);

    let absolute_root = absolute_root.to_path_buf();
    let pack_set = PackSet::build(included_packs, owning_package_yml_for_file);

    let cache_directory = absolute_root.join(raw_config.cache_directory);
    let cache_enabled = raw_config.cache;
    let experimental_parser = raw_config.experimental_parser;

    let layers = Layers {
        layers: raw_config.architecture_layers,
    };

    debug!("Finished building configuration");

    Configuration {
        included_files,
        absolute_root,
        cache_enabled,
        cache_directory,
        pack_set,
        layers,
        experimental_parser,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::packs::{
        configuration,
        pack::{CheckerSetting, Pack},
        PackageTodo,
    };

    use pretty_assertions::assert_eq;

    #[test]
    fn default_options() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let actual = configuration::get(&absolute_root);
        assert_eq!(actual.absolute_root, absolute_root);

        let expected_included_files = vec![
            absolute_root.join("packs/bar/app/services/bar.rb"),
            absolute_root.join("packs/foo/app/services/foo.rb"),
            absolute_root.join("packs/foo/app/services/foo/bar.rb"),
            absolute_root.join("packs/foo/app/views/foo.erb"),
            absolute_root.join("packs/baz/app/services/baz.rb"),
            absolute_root.join("packs/bar/app/models/concerns/some_concern.rb"),
            absolute_root.join("app/services/some_root_class.rb"),
        ]
        .into_iter()
        .collect::<HashSet<PathBuf>>();
        assert_eq!(actual.included_files, expected_included_files);

        let expected_packs = vec![
            Pack {
                enforce_dependencies: CheckerSetting::False,
                enforce_privacy: CheckerSetting::True,
                enforce_visibility: CheckerSetting::False,
                enforce_architecture: CheckerSetting::False,
                yml: absolute_root.join("packs/bar/package.yml"),
                name: String::from("packs/bar"),
                relative_path: PathBuf::from("packs/bar"),
                dependencies: HashSet::new(),
                visible_to: HashSet::new(),
                package_todo: PackageTodo::default(),
                ignored_dependencies: HashSet::new(),
                ignored_private_constants: HashSet::new(),
                private_constants: HashSet::new(),
                public_folder: PathBuf::from("packs/bar/app/public"),
                layer: None,
            },
            Pack {
                enforce_dependencies: CheckerSetting::False,
                enforce_privacy: CheckerSetting::False,
                enforce_visibility: CheckerSetting::False,
                enforce_architecture: CheckerSetting::False,
                yml: absolute_root.join("packs/baz/package.yml"),
                name: String::from("packs/baz"),
                relative_path: PathBuf::from("packs/baz"),
                dependencies: HashSet::new(),
                visible_to: HashSet::new(),
                package_todo: PackageTodo::default(),
                ignored_dependencies: HashSet::new(),
                ignored_private_constants: HashSet::new(),
                private_constants: HashSet::new(),
                public_folder: PathBuf::from("packs/baz/app/public"),
                layer: None,
            },
            Pack {
                enforce_dependencies: CheckerSetting::True,
                enforce_privacy: CheckerSetting::True,
                enforce_visibility: CheckerSetting::False,
                enforce_architecture: CheckerSetting::False,
                yml: absolute_root.join("packs/foo/package.yml"),
                name: String::from("packs/foo"),
                relative_path: PathBuf::from("packs/foo"),
                dependencies: HashSet::from_iter(vec![String::from(
                    "packs/baz",
                )]),
                visible_to: HashSet::new(),
                package_todo: PackageTodo::default(),
                ignored_dependencies: HashSet::new(),
                ignored_private_constants: HashSet::new(),
                private_constants: HashSet::new(),
                public_folder: PathBuf::from("packs/foo/app/public"),
                layer: None,
            },
            Pack {
                enforce_dependencies: CheckerSetting::False,
                enforce_privacy: CheckerSetting::False,
                enforce_visibility: CheckerSetting::False,
                enforce_architecture: CheckerSetting::False,
                yml: absolute_root.join("package.yml"),
                name: String::from("."),
                relative_path: PathBuf::from("."),
                dependencies: HashSet::new(),
                visible_to: HashSet::new(),
                package_todo: PackageTodo::default(),
                ignored_dependencies: HashSet::new(),
                ignored_private_constants: HashSet::new(),
                private_constants: HashSet::new(),
                public_folder: PathBuf::from("./app/public"),
                layer: None,
            },
        ];

        assert_eq!(expected_packs, actual.pack_set.packs);

        assert!(!actual.cache_enabled)
    }

    #[test]
    fn filtered_absolute_paths_with_nonempty_input_paths() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let configuration = configuration::get(&absolute_root);
        let actual_paths = configuration.intersect_files(vec![
            String::from("packs/foo/app/services/foo.rb"),
            String::from("scripts/my_script.rb"),
            String::from("packs/bar/app/services/bar.rb"),
            String::from("vendor/some_gem/foo.rb"),
        ]);
        let expected_paths = vec![
            absolute_root.join("packs/bar/app/services/bar.rb"),
            absolute_root.join("packs/foo/app/services/foo.rb"),
        ]
        .into_iter()
        .collect::<HashSet<PathBuf>>();
        assert_eq!(actual_paths, expected_paths);
    }

    #[test]
    fn filtered_absolute_paths_with_empty_input_paths() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let configuration = configuration::get(&absolute_root);
        let actual_paths = configuration.intersect_files(vec![]);
        let expected_paths = vec![
            absolute_root.join("packs/bar/app/services/bar.rb"),
            absolute_root.join("packs/foo/app/services/foo.rb"),
            absolute_root.join("packs/foo/app/services/foo/bar.rb"),
            absolute_root.join("packs/foo/app/views/foo.erb"),
            absolute_root.join("packs/baz/app/services/baz.rb"),
            absolute_root.join("packs/bar/app/models/concerns/some_concern.rb"),
            absolute_root.join("app/services/some_root_class.rb"),
        ]
        .into_iter()
        .collect::<HashSet<PathBuf>>();
        assert_eq!(actual_paths, expected_paths);
    }

    #[test]
    fn filtered_absolute_paths_with_directory_input_paths() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let configuration = configuration::get(&absolute_root);
        let actual_paths =
            configuration.intersect_files(vec![String::from("packs/bar")]);
        let expected_paths = vec![
            absolute_root.join("packs/bar/app/services/bar.rb"),
            absolute_root.join("packs/bar/app/models/concerns/some_concern.rb"),
        ]
        .into_iter()
        .collect::<HashSet<PathBuf>>();
        assert_eq!(actual_paths, expected_paths);
    }
}
