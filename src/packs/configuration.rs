use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
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
    vec![String::from("**/*.{rb,rake,erb}")]
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

pub struct Configuration {
    pub included_files: Vec<PathBuf>,
    pub absolute_root: PathBuf,
    pub package_paths: Vec<PathBuf>,
    pub cache_enabled: bool,
}

fn get_included_files(
    absolute_root: &Path,
    raw: &RawConfiguration,
) -> Vec<PathBuf> {
    // Adding a `!` to the beginning of a glob pattern negates it.
    let exclude_patterns = raw.exclude.iter().map(|p| format!("!{}", p));

    let mut combined_patterns = raw.include.clone();
    combined_patterns.extend(exclude_patterns);

    let included_files: Vec<PathBuf> =
        globwalk::GlobWalkerBuilder::from_patterns(
            absolute_root,
            &combined_patterns,
        )
        .build()
        .expect("Could not build glob walker")
        .filter_map(Result::ok)
        .map(|x| x.into_path())
        .sorted() // Make output deterministic
        .collect();

    included_files
}

fn get_package_paths(
    absolute_root: &Path,
    raw: &RawConfiguration,
) -> Vec<PathBuf> {
    let package_yml_paths: Vec<String> = raw
        .package_paths
        .clone()
        .into_iter()
        .map(|p| format!("{}package.yml", p))
        .collect();

    let package_paths: Vec<PathBuf> =
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

pub(crate) fn get(absolute_root: PathBuf) -> Configuration {
    let absolute_path_to_packwerk_yml = absolute_root.join("packwerk.yml");
    let mut file = File::open(absolute_path_to_packwerk_yml.clone())
        .unwrap_or_else(|e| {
            panic!(
                "Could not open packwerk.yml at: {} due to error: {:?}",
                absolute_path_to_packwerk_yml.display(),
                e
            )
        });

    let mut contents = String::new();
    std::io::Read::read_to_string(&mut file, &mut contents).unwrap_or_else(
        |e| {
            panic!(
                "Could not read packwerk.yml at: {} due to error: {:?}",
                absolute_path_to_packwerk_yml.display(),
                e
            )
        },
    );

    let raw_config: RawConfiguration = serde_yaml::from_str(&contents)
        .unwrap_or_else(|e| {
            panic!(
                "Could not parse packwerk.yml at: {} due to error: {:?}",
                absolute_path_to_packwerk_yml.display(),
                e
            )
        });

    Configuration {
        included_files: get_included_files(&absolute_root, &raw_config),
        absolute_root: absolute_root.clone(),
        package_paths: get_package_paths(&absolute_root, &raw_config),
        cache_enabled: raw_config.cache,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::packs::configuration;

    #[test]
    fn default_options() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let actual = configuration::get(absolute_root.clone());
        assert_eq!(actual.absolute_root, absolute_root);
        assert_eq!(
            actual.included_files,
            vec![
                absolute_root.join("packs/bar/app/services/bar.rb"),
                absolute_root.join("packs/foo/app/services/foo.rb"),
                absolute_root.join("packs/foo/app/views/foo.erb")
            ]
        );

        assert_eq!(
            actual.package_paths,
            vec![
                absolute_root.join("packs/foo/package.yml"),
                absolute_root.join("packs/bar/package.yml"),
                absolute_root.join("package.yml"),
            ]
        );

        assert!(actual.cache_enabled)
    }
}
