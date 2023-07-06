use std::{fs::File, path::Path};

use serde::{Deserialize, Serialize};

const CONFIG_FILE_NAME: &str = "packwerk.yml";

pub(crate) fn get(absolute_root: &Path) -> RawConfiguration {
    let absolute_path_to_packwerk_yml = absolute_root.join(CONFIG_FILE_NAME);

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
        std::io::Read::read_to_string(&mut file, &mut contents).unwrap_or_else(
            |e| {
                panic!(
                    "Could not read packwerk.yml at: {} due to error: {:?}",
                    absolute_path_to_packwerk_yml.display(),
                    e
                )
            },
        );

        serde_yaml::from_str(&contents).unwrap_or_else(|e| {
            panic!(
                "Could not parse packwerk.yml at: {} due to error: {:?}",
                absolute_path_to_packwerk_yml.display(),
                e
            )
        })
    } else {
        RawConfiguration::default()
    }
}
// See: Setting up the configuration file
// https://github.com/Shopify/packwerk/blob/main/USAGE.md#setting-up-the-configuration-file
#[derive(Debug, Deserialize, Serialize, Default)]
pub(crate) struct RawConfiguration {
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

    // Architecture layers
    #[serde(default)]
    pub architecture_layers: Vec<String>,

    // Experimental parser
    #[serde(default)]
    pub experimental_parser: bool,
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

// Add a test that the default RawConfiguration tmp directory is tmp/cache/packwerk
// Add a test that the default RawConfiguration cache is true
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let raw_configuration = RawConfiguration::default();

        assert_eq!(raw_configuration.cache, true);
        assert_eq!(raw_configuration.cache_directory, "tmp/cache/packwerk");
    }
}
