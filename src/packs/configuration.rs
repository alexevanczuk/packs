use itertools::Itertools;
use jwalk::WalkDir;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::File,
    path::{Path, PathBuf},
};

// See: Setting up the configuration file
// https://github.com/Shopify/packwerk/blob/main/USAGE.md#setting-up-the-configuration-file
#[derive(Debug, Deserialize, Serialize, Default)]
struct RawConfiguration {
    // List of patterns for folder paths to include
    #[serde(default = "default_include")]
    include: Vec<String>,

    // List of patterns for folder paths to exclude
    #[serde(default = "default_exclude")]
    exclude: Vec<String>,

    // Patterns to find package configuration files
    #[serde(default = "default_package_paths")]
    package_paths: Vec<String>,

    // List of custom associations, if any
    #[serde(default = "default_custom_associations")]
    custom_associations: Vec<String>,

    // Whether or not you want the cache enabled
    #[serde(default = "default_cache")]
    cache: bool,

    // Where you want the cache to be stored
    #[serde(default = "default_cache_directory")]
    cache_directory: String,
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
    vec![String::from("**/")]
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
    pub package_paths: HashSet<PathBuf>,
    pub cache_enabled: bool,
    pub cache_directory: PathBuf,
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

fn matches_globs(path: &Path, globs: &[String]) -> bool {
    globs
        .iter()
        .any(|glob| glob_match::glob_match(glob, path.to_str().unwrap()))
}

// We use jwalk to walk directories in parallel and compare them to the `include` and `exclude` patterns
// specified in the `RawConfiguration`
// https://docs.rs/jwalk/0.8.1/jwalk/struct.WalkDirGeneric.html#method.process_read_dir
fn get_included_files(
    absolute_root: &Path,
    raw: &RawConfiguration,
) -> HashSet<PathBuf> {
    let mut included_paths: HashSet<PathBuf> = HashSet::new();

    for entry in WalkDir::new(absolute_root) {
        let absolute_path = entry.unwrap().path();
        let relative_path = absolute_path
            .strip_prefix(absolute_root)
            .unwrap()
            .to_owned();

        if matches_globs(&relative_path, &raw.include)
            && !matches_globs(&relative_path, &raw.exclude)
        {
            included_paths.insert(absolute_path);
        }
    }

    included_paths
}

fn get_package_paths(
    absolute_root: &Path,
    raw: &RawConfiguration,
) -> HashSet<PathBuf> {
    let package_yml_paths: Vec<String> = raw
        .package_paths
        .clone()
        .into_iter()
        .map(|p| format!("{}package.yml", p))
        .collect();

    let package_paths: HashSet<PathBuf> =
        globwalk::GlobWalkerBuilder::from_patterns(
            absolute_root,
            &package_yml_paths,
        )
        .build()
        .expect("Could not build glob walker")
        .filter_map(Result::ok)
        .map(|x| x.into_path())
        // .sorted() // Make output deterministic
        .sorted_by(|packa, packb| {
            Ord::cmp(
                &packb.to_string_lossy().len(),
                &packa.to_string_lossy().len(),
            )
        })
        .collect();

    package_paths
}

pub(crate) fn get(absolute_root: &Path) -> Configuration {
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

    let included_files = get_included_files(absolute_root, &raw_config);

    Configuration {
        included_files,
        absolute_root: absolute_root.to_path_buf(),
        package_paths: get_package_paths(absolute_root, &raw_config),
        cache_enabled: raw_config.cache,
        cache_directory: absolute_root.join(raw_config.cache_directory),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::packs::configuration;

    #[test]
    fn default_options() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let actual = configuration::get(&absolute_root);
        assert_eq!(actual.absolute_root, absolute_root);

        let expected_included_files = vec![
            absolute_root.join("packs/bar/app/services/bar.rb"),
            absolute_root.join("packs/foo/app/services/foo.rb"),
            absolute_root.join("packs/foo/app/views/foo.erb"),
            absolute_root.join("packs/baz/app/services/baz.rb"),
        ]
        .into_iter()
        .collect::<HashSet<PathBuf>>();
        assert_eq!(actual.included_files, expected_included_files);

        let expected_package_paths = vec![
            absolute_root.join("packs/foo/package.yml"),
            absolute_root.join("packs/bar/package.yml"),
            absolute_root.join("packs/baz/package.yml"),
            absolute_root.join("package.yml"),
        ]
        .into_iter()
        .collect::<HashSet<PathBuf>>();
        assert_eq!(actual.package_paths, expected_package_paths);

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
            absolute_root.join("packs/foo/app/views/foo.erb"),
            absolute_root.join("packs/baz/app/services/baz.rb"),
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
        let expected_paths =
            vec![absolute_root.join("packs/bar/app/services/bar.rb")]
                .into_iter()
                .collect::<HashSet<PathBuf>>();
        assert_eq!(actual_paths, expected_paths);
    }
}
