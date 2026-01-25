use super::caching::{
    cache::Cache, create_cache_dir_idempotently, noop_cache::NoopCache,
    per_file_cache::PerFileCache,
};
use super::checker::layer::Layers;
use super::file_utils::{
    file_content_digest, user_inputted_paths_to_absolute_filepaths,
};

use super::{
    constant_resolver::ConstantResolverConfiguration, raw_configuration,
    raw_configuration::RawConfiguration, walk_directory,
    walk_directory::WalkDirectoryResult, PackSet,
};

use std::collections::HashMap;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use tracing::debug;
use walk_directory::walk_directory;

#[derive(Debug)]
pub struct Configuration {
    pub included_files: HashSet<PathBuf>,
    pub input_files_count: usize, // Helpful for optimizations in privacy chcker
    pub absolute_root: PathBuf,
    pub cache_enabled: bool,
    pub cache_directory: PathBuf,
    pub config_file_path: Option<PathBuf>,
    pub pack_set: PackSet,
    pub layers: Layers,
    pub experimental_parser: bool,
    pub ignored_definitions: HashMap<String, HashSet<PathBuf>>,
    pub autoload_roots: HashMap<PathBuf, String>,
    pub inflections_path: PathBuf,
    pub custom_associations: Vec<String>,
    pub stdin_file_path: Option<PathBuf>,
    // Note that it'd probably be better to use the logger library, `tracing` (see logger.rs)
    // and configure logging in one place. As the complexity of how/why we want to see different logs
    // grows, we can refactor this.
    pub print_files: bool,
    pub packs_first_mode: bool,
    pub ignore_recorded_violations: bool,
    pub disable_enforce_dependencies: bool,
    pub disable_enforce_folder_privacy: bool,
    pub disable_enforce_layers: bool,
    pub disable_enforce_privacy: bool,
    pub disable_enforce_visibility: bool,
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

    pub(crate) fn get_cache(&self) -> Box<dyn Cache + Send + Sync> {
        if self.cache_enabled {
            let parser_dir = if self.experimental_parser {
                "experimental"
            } else {
                "zeitwerk"
            };

            // Include config file digest in cache path so config changes invalidate cache
            let config_digest_prefix = self
                .config_file_path
                .as_ref()
                .and_then(|path| file_content_digest(path).ok())
                .map(|digest| digest[..8].to_string())
                .unwrap_or_else(|| "no_config".to_string());

            let parser_cache_dir = self.cache_directory.join(parser_dir);
            let cache_dir = parser_cache_dir.join(&config_digest_prefix);

            // Clean up old cache directories with different config digests
            if let Ok(entries) = std::fs::read_dir(&parser_cache_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir()
                        && path.file_name()
                            != Some(std::ffi::OsStr::new(&config_digest_prefix))
                    {
                        let _ = std::fs::remove_dir_all(&path);
                    }
                }
            }

            create_cache_dir_idempotently(&cache_dir);

            Box::new(PerFileCache { cache_dir })
        } else {
            Box::new(NoopCache {})
        }
    }

    pub(crate) fn constant_resolver_configuration(
        &self,
    ) -> ConstantResolverConfiguration {
        ConstantResolverConfiguration {
            absolute_root: &self.absolute_root,
            cache_directory: &self.cache_directory,
            cache_enabled: self.cache_enabled,
            autoload_roots: &self.autoload_roots,
            inflections_path: &self.inflections_path,
        }
    }
}

pub(crate) fn get(
    absolute_root: &Path,
    input_files_count: &usize,
) -> anyhow::Result<Configuration> {
    debug!("Beginning to build configuration");

    let (raw_config, config_file_path) = raw_configuration::get(absolute_root)?;
    let walk_directory_result =
        walk_directory(absolute_root.to_path_buf(), &raw_config)?;

    from_raw(
        absolute_root,
        raw_config,
        config_file_path,
        walk_directory_result,
        input_files_count,
    )
}

pub(crate) fn from_raw(
    absolute_root: &Path,
    raw_config: RawConfiguration,
    config_file_path: Option<PathBuf>,
    walk_directory_result: WalkDirectoryResult,
    input_files_count: &usize,
) -> anyhow::Result<Configuration> {
    let WalkDirectoryResult {
        included_files,
        included_packs,
        owning_package_yml_for_file,
    } = walk_directory_result;

    let absolute_root = absolute_root.to_path_buf();
    let pack_set = PackSet::build(included_packs, owning_package_yml_for_file)?;

    let cache_directory = absolute_root.join(raw_config.cache_directory);
    let cache_enabled = raw_config.cache;
    let experimental_parser = raw_config.experimental_parser;

    let layers = Layers {
        layers: raw_config.layers,
    };

    let ignored_definitions = raw_config.ignored_definitions;
    let autoload_roots: HashMap<PathBuf, String> = raw_config.autoload_roots;

    let packs_first_mode = raw_config.packs_first_mode;

    let inflections_path = absolute_root.join(
        raw_config
            .inflections_path
            .unwrap_or(PathBuf::from("config/initializers/inflections.rb")),
    );

    let custom_associations = raw_config
        .custom_associations
        .iter()
        // In packwerk, custom_associations are an array of symbols. We strip the leading : so this configuration is compatible with the rust implementation.
        .map(|a| a.trim_start_matches(':').to_owned())
        .collect();

    debug!("Finished building configuration");

    Ok(Configuration {
        included_files,
        input_files_count: input_files_count.to_owned(),
        absolute_root,
        cache_enabled,
        cache_directory,
        config_file_path,
        pack_set,
        layers,
        experimental_parser,
        ignored_definitions,
        autoload_roots,
        inflections_path,
        custom_associations,
        stdin_file_path: None,
        print_files: false,
        packs_first_mode,
        ignore_recorded_violations: false,
        disable_enforce_dependencies: false,
        disable_enforce_folder_privacy: false,
        disable_enforce_layers: false,
        disable_enforce_privacy: false,
        disable_enforce_visibility: false,
    })
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
        let actual = configuration::get(&absolute_root, &0).unwrap();
        assert_eq!(actual.absolute_root, absolute_root);

        let expected_included_files = vec![
            absolute_root.join("frontend/ui_helper.rb"),
            absolute_root.join("packs/bar/app/services/bar.rb"),
            absolute_root.join("packs/foo/app/services/foo.rb"),
            absolute_root.join("packs/foo/app/services/foo/bar.rb"),
            absolute_root.join("packs/foo/app/views/foo.erb"),
            absolute_root.join("packs/baz/app/services/baz.rb"),
            absolute_root.join("packs/bar/app/models/concerns/some_concern.rb"),
            absolute_root.join("app/services/some_root_class.rb"),
            absolute_root.join("app/company_data/widget.rb"),
        ]
        .into_iter()
        .collect::<HashSet<PathBuf>>();
        assert_eq!(actual.included_files, expected_included_files);

        let expected_packs = vec![
            Pack {
                enforce_dependencies: None,
                enforce_privacy: Some(CheckerSetting::True),
                enforce_visibility: None,
                enforce_folder_privacy: None,
                enforce_folder_visibility: None,
                enforce_layers: None,
                owner: None,
                yml: absolute_root.join("packs/bar/package.yml"),
                name: String::from("packs/bar"),
                relative_path: PathBuf::from("packs/bar"),
                dependencies: HashSet::new(),
                visible_to: None,
                package_todo: PackageTodo::default(),
                ignored_dependencies: HashSet::new(),
                ignored_private_constants: HashSet::new(),
                private_constants: HashSet::new(),
                public_folder: None,
                layer: None,
                client_keys: HashMap::new(),
                enforcement_globs_ignore: None,
            },
            Pack {
                enforce_dependencies: None,
                enforce_privacy: None,
                enforce_visibility: None,
                enforce_folder_privacy: None,
                enforce_folder_visibility: None,
                enforce_layers: None,
                owner: None,
                yml: absolute_root.join("packs/baz/package.yml"),
                name: String::from("packs/baz"),
                relative_path: PathBuf::from("packs/baz"),
                dependencies: HashSet::new(),
                visible_to: None,
                package_todo: PackageTodo::default(),
                ignored_dependencies: HashSet::new(),
                ignored_private_constants: HashSet::new(),
                private_constants: HashSet::new(),
                public_folder: None,
                layer: None,
                client_keys: HashMap::new(),
                enforcement_globs_ignore: None,
            },
            Pack {
                enforce_dependencies: Some(CheckerSetting::True),
                enforce_privacy: Some(CheckerSetting::True),
                enforce_visibility: None,
                enforce_folder_privacy: None,
                enforce_folder_visibility: None,
                enforce_layers: None,
                owner: None,
                yml: absolute_root.join("packs/foo/package.yml"),
                name: String::from("packs/foo"),
                relative_path: PathBuf::from("packs/foo"),
                dependencies: HashSet::from_iter(vec![String::from(
                    "packs/baz",
                )]),
                visible_to: None,
                package_todo: PackageTodo::default(),
                ignored_dependencies: HashSet::new(),
                ignored_private_constants: HashSet::new(),
                private_constants: HashSet::new(),
                public_folder: None,

                layer: None,
                client_keys: HashMap::new(),
                enforcement_globs_ignore: None,
            },
            Pack {
                enforce_dependencies: None,
                enforce_privacy: None,
                enforce_visibility: None,
                enforce_folder_privacy: None,
                enforce_folder_visibility: None,
                enforce_layers: None,
                owner: None,
                yml: absolute_root.join("package.yml"),
                name: String::from("."),
                relative_path: PathBuf::from("."),
                dependencies: HashSet::new(),
                visible_to: None,
                package_todo: PackageTodo::default(),
                ignored_dependencies: HashSet::new(),
                ignored_private_constants: HashSet::new(),
                private_constants: HashSet::new(),
                public_folder: None,
                layer: None,
                client_keys: HashMap::new(),
                enforcement_globs_ignore: None,
            },
        ];

        assert_eq!(expected_packs, actual.pack_set.packs);

        assert!(!actual.cache_enabled)
    }

    #[test]
    fn filtered_absolute_paths_with_nonempty_input_paths() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let configuration = configuration::get(&absolute_root, &0).unwrap();
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
        let configuration = configuration::get(&absolute_root, &0).unwrap();
        let actual_paths = configuration.intersect_files(vec![]);
        let expected_paths = vec![
            absolute_root.join("frontend/ui_helper.rb"),
            absolute_root.join("packs/bar/app/services/bar.rb"),
            absolute_root.join("packs/foo/app/services/foo.rb"),
            absolute_root.join("packs/foo/app/services/foo/bar.rb"),
            absolute_root.join("packs/foo/app/views/foo.erb"),
            absolute_root.join("packs/baz/app/services/baz.rb"),
            absolute_root.join("packs/bar/app/models/concerns/some_concern.rb"),
            absolute_root.join("app/services/some_root_class.rb"),
            absolute_root.join("app/company_data/widget.rb"),
        ]
        .into_iter()
        .collect::<HashSet<PathBuf>>();
        assert_eq!(actual_paths, expected_paths);
    }

    #[test]
    fn filtered_absolute_paths_with_directory_input_paths() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let configuration = configuration::get(&absolute_root, &0).unwrap();
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

    #[test]
    fn with_symbols_as_custom_associations() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let raw = RawConfiguration {
            custom_associations: vec![":my_association".to_owned()],
            ..RawConfiguration::default()
        };

        let included_packs: HashSet<Pack> = vec![Pack {
            name: String::from("."),
            ..Pack::default()
        }]
        .into_iter()
        .collect();
        let walk_directory_result = WalkDirectoryResult {
            included_files: Default::default(),
            included_packs,
            owning_package_yml_for_file: Default::default(),
        };

        let configuration = configuration::from_raw(
            &absolute_root,
            raw,
            None,
            walk_directory_result,
            &0,
        )
        .unwrap();
        let actual_associations = configuration.custom_associations;
        let expected_paths = vec!["my_association".to_owned()];

        assert_eq!(actual_associations, expected_paths);
    }

    #[test]
    fn cache_directory_includes_config_digest() {
        use tempfile::TempDir;

        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let mut config = configuration::get(&absolute_root, &0).unwrap();

        // Use temp directory to avoid conflicts with other tests
        let temp_dir = TempDir::new().unwrap();
        config.cache_directory = temp_dir.path().to_path_buf();
        config.cache_enabled = true;

        // Config file should be set
        assert!(config.config_file_path.is_some());

        // Get the cache and check it was created with a digest subdirectory
        let _cache = config.get_cache();
        let parser_dir = config.cache_directory.join("zeitwerk");

        // Should have a subdirectory with 8-char hex name (config digest)
        let entries: Vec<_> = std::fs::read_dir(&parser_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();

        assert_eq!(entries.len(), 1);
        let dir_name = entries[0].file_name();
        let dir_name_str = dir_name.to_str().unwrap();
        assert_eq!(dir_name_str.len(), 8);
        assert!(dir_name_str.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn cache_cleanup_removes_old_digest_directories() {
        use tempfile::TempDir;

        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let mut config = configuration::get(&absolute_root, &0).unwrap();

        // Use temp directory to avoid conflicts with other tests
        let temp_dir = TempDir::new().unwrap();
        config.cache_directory = temp_dir.path().to_path_buf();
        config.cache_enabled = true;

        let parser_dir = config.cache_directory.join("zeitwerk");

        // Create a fake old cache directory
        let old_cache_dir = parser_dir.join("deadbeef");
        std::fs::create_dir_all(&old_cache_dir).unwrap();
        std::fs::write(old_cache_dir.join("test_file"), "test").unwrap();

        // Getting cache should clean up the old directory
        let _cache = config.get_cache();

        // Old directory should be gone
        assert!(!old_cache_dir.exists());

        // But new directory should exist
        let entries: Vec<_> = std::fs::read_dir(&parser_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();
        assert_eq!(entries.len(), 1);
    }
}
