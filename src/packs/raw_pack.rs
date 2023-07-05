use std::collections::HashSet;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct RawPack {
    #[serde(default)]
    pub dependencies: HashSet<String>,
    #[serde(default)]
    pub ignored_dependencies: HashSet<String>,
    #[serde(default)]
    pub visible_to: HashSet<String>,
    #[serde(default)]
    pub ignored_private_constants: HashSet<String>,
    #[serde(default = "default_public_folder")]
    pub public_folder: String,
    #[serde(default)]
    pub layer: Option<String>,
    #[serde(default = "default_checker_setting")]
    pub enforce_dependencies: String,
    #[serde(default = "default_checker_setting")]
    pub enforce_privacy: String,
    #[serde(default = "default_checker_setting")]
    pub enforce_visibility: String,
    #[serde(default = "default_checker_setting")]
    pub enforce_architecture: String,
}

fn default_checker_setting() -> String {
    "false".to_string()
}

fn default_public_folder() -> String {
    "app/public".to_string()
}
