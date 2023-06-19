use super::PackSet;
use crate::packs::Pack;
use crate::packs::{
    parser::ruby::packwerk::constant_resolver::ConstantResolver, walk_directory,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::File,
    path::{Path, PathBuf},
};
use tracing::debug;
use walk_directory::walk_directory;

// See: Setting up the configuration file
// https://github.com/Shopify/packwerk/blob/main/USAGE.md#setting-up-the-configuration-file
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct RawConfiguration {
    // List of patterns for folder paths to include
    #[serde(default = "default_include")]
    pub include: Vec<String>,

    // List of patterns for folder paths to exclude
    #[serde(default = "default_exclude")]
    pub exclude: Vec<String>,

    // Patterns to find package configuration files
    #[serde(default = "default_package_paths")]
    pub package_paths: Vec<String>,

    // List of custom associations, if any
    #[serde(default = "default_custom_associations")]
    pub custom_associations: Vec<String>,

    // Whether or not you want the cache enabled
    #[serde(default = "default_cache")]
    pub cache: bool,

    // Where you want the cache to be stored
    #[serde(default = "default_cache_directory")]
    pub cache_directory: String,

    // Autoload paths used to resolve constants
    #[serde(default)]
    pub autoload_paths: Option<Vec<String>>,
}

fn default_include() -> Vec<String> {
    vec![
        String::from("**/*.rb"),
        String::from("**/*.rake"),
        String::from("**/*.erb"),
    ]
}

fn default_exclude() -> Vec<String> {
    vec![String::from("{bin,node_modules,script,tmp,vendor}/**/*")]
}

fn default_package_paths() -> Vec<String> {
    vec![String::from("**/*")]
}

fn default_custom_associations() -> Vec<String> {
    vec![]
}

fn default_cache() -> bool {
    true
}

fn default_cache_directory() -> String {
    String::from("tmp/cache/packwerk")
}

#[derive(Default)]
pub struct Configuration {
    pub included_files: HashSet<PathBuf>,
    pub absolute_root: PathBuf,
    pub cache_enabled: bool,
    pub cache_directory: PathBuf,
    pub constant_resolver: ConstantResolver,
    pub pack_set: PackSet,
}

impl Configuration {
    pub(crate) fn intersect_files(
        &self,
        input_files: Vec<String>,
    ) -> HashSet<PathBuf> {
        if input_files.is_empty() {
            self.included_files.clone()
        } else {
            let input_paths = input_files
                .iter()
                .map(PathBuf::from)
                .flat_map(|p| {
                    if p.is_absolute() {
                        vec![p]
                    } else {
                        let absolute_path = self.absolute_root.join(&p);
                        if absolute_path.is_dir() {
                            glob::glob(
                                absolute_path.join("**/*.*").to_str().unwrap(),
                            )
                            .expect("Failed to read glob pattern")
                            .filter_map(Result::ok)
                            .collect::<Vec<_>>()
                        } else {
                            vec![absolute_path]
                        }
                    }
                })
                .collect::<HashSet<_>>();

            self.included_files
                .intersection(&input_paths)
                .cloned()
                .collect::<HashSet<PathBuf>>()
        }
    }
}

pub(crate) fn get(absolute_root: &Path) -> Configuration {
    debug!("Beginning to build configuration");
    let absolute_path_to_packwerk_yml = absolute_root.join("packwerk.yml");

    let raw_config: RawConfiguration =
        if absolute_path_to_packwerk_yml.exists() {
            let mut file = File::open(absolute_path_to_packwerk_yml.clone())
                .unwrap_or_else(|e| {
                    panic!(
                        "Could not open packwerk.yml at: {} due to error: {:?}",
                        absolute_path_to_packwerk_yml.display(),
                        e
                    )
                });

            let mut contents = String::new();
            std::io::Read::read_to_string(&mut file, &mut contents)
                .unwrap_or_else(|e| {
                    panic!(
                        "Could not read packwerk.yml at: {} due to error: {:?}",
                        absolute_path_to_packwerk_yml.display(),
                        e
                    )
                });

            serde_yaml::from_str(&contents).unwrap_or_else(|e| {
                panic!(
                    "Could not parse packwerk.yml at: {} due to error: {:?}",
                    absolute_path_to_packwerk_yml.display(),
                    e
                )
            })
        } else {
            RawConfiguration::default()
        };

    debug!("Beginning directory walk");

    let (included_files, unsorted_packs) =
        walk_directory(absolute_root.to_path_buf(), &raw_config);
    debug!("Finished directory walk");

    let absolute_root = absolute_root.to_path_buf();
    let pack_set = PackSet::build(unsorted_packs);

    let autoload_paths = get_autoload_paths(&pack_set.packs);

    let cache_directory = absolute_root.join(raw_config.cache_directory);
    let cache_enabled = raw_config.cache;
    let constant_resolver =
        ConstantResolver::create(&absolute_root, autoload_paths);

    debug!("Finished building configuration");

    Configuration {
        included_files,
        absolute_root,
        cache_enabled,
        cache_directory,
        constant_resolver,
        pack_set,
    }
}

fn get_autoload_paths(packs: &Vec<Pack>) -> Vec<PathBuf> {
    let mut autoload_paths: Vec<PathBuf> = Vec::new();

    debug!("Getting autoload paths");
    for pack in packs {
        // App paths
        let app_paths = pack.yml.parent().unwrap().join("app").join("*");
        let app_glob_pattern = app_paths.to_str().unwrap();
        process_glob_pattern(app_glob_pattern, &mut autoload_paths);

        // Concerns paths
        let concerns_paths = pack
            .yml
            .parent()
            .unwrap()
            .join("app")
            .join("*")
            .join("concerns");
        let concerns_glob_pattern = concerns_paths.to_str().unwrap();

        process_glob_pattern(concerns_glob_pattern, &mut autoload_paths);
    }

    debug!("Finished getting autoload paths");

    autoload_paths
}

fn process_glob_pattern(pattern: &str, paths: &mut Vec<PathBuf>) {
    for path in glob::glob(pattern)
        .expect("Failed to read glob pattern")
        .flatten()
    {
        paths.push(path);
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::packs::{configuration, CheckerSetting, PackageTodo};

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

                yml: absolute_root.join("packs/bar/package.yml"),
                name: String::from("packs/bar"),
                relative_path: PathBuf::from("packs/bar"),
                dependencies: HashSet::new(),
                package_todo: PackageTodo::default(),
                ignored_dependencies: HashSet::new(),
            },
            Pack {
                enforce_dependencies: CheckerSetting::False,

                yml: absolute_root.join("packs/baz/package.yml"),
                name: String::from("packs/baz"),
                relative_path: PathBuf::from("packs/baz"),
                dependencies: HashSet::new(),
                package_todo: PackageTodo::default(),
                ignored_dependencies: HashSet::new(),
            },
            Pack {
                enforce_dependencies: CheckerSetting::True,

                yml: absolute_root.join("packs/foo/package.yml"),
                name: String::from("packs/foo"),
                relative_path: PathBuf::from("packs/foo"),
                dependencies: HashSet::from_iter(vec![String::from(
                    "packs/baz",
                )]),
                package_todo: PackageTodo::default(),
                ignored_dependencies: HashSet::new(),
            },
            Pack {
                enforce_dependencies: CheckerSetting::False,
                yml: absolute_root.join("package.yml"),
                name: String::from("."),
                relative_path: PathBuf::from("."),
                dependencies: HashSet::new(),
                package_todo: PackageTodo::default(),
                ignored_dependencies: HashSet::new(),
            },
        ];

        let mut expected_autoload_paths = vec![
            PathBuf::from("tests/fixtures/simple_app/app/services"),
            PathBuf::from("tests/fixtures/simple_app/packs/bar/app/models"),
            PathBuf::from(
                "tests/fixtures/simple_app/packs/bar/app/models/concerns",
            ),
            PathBuf::from("tests/fixtures/simple_app/packs/bar/app/services"),
            PathBuf::from("tests/fixtures/simple_app/packs/baz/app/services"),
            PathBuf::from("tests/fixtures/simple_app/packs/foo/app/services"),
            PathBuf::from("tests/fixtures/simple_app/packs/foo/app/views"),
        ];
        expected_autoload_paths.sort();
        let mut actual_autoload_paths =
            actual.constant_resolver.autoload_paths.clone();
        actual_autoload_paths.sort();

        assert_eq!(expected_autoload_paths, actual_autoload_paths);
        assert_eq!(expected_packs, actual.pack_set.packs);

        assert!(actual.cache_enabled)
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
