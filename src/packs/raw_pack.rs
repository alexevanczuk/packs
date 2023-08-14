use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::Path,
};

use serde::{Deserialize, Serialize, Serializer};
use serde_yaml::Value;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct RawPack {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_checker_setting"
    )]
    pub enforce_dependencies: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_checker_setting"
    )]
    pub enforce_privacy: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_checker_setting"
    )]
    pub enforce_visibility: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_checker_setting"
    )]
    pub enforce_architecture: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    #[serde(
        default,
        skip_serializing_if = "HashSet::is_empty",
        serialize_with = "serialize_sorted_hashset_of_strings"
    )]
    pub dependencies: HashSet<String>,
    #[serde(
        default,
        skip_serializing_if = "HashSet::is_empty",
        serialize_with = "serialize_sorted_hashset_of_strings"
    )]
    pub ignored_dependencies: HashSet<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_sorted_option_hashset_of_strings"
    )]
    pub visible_to: Option<HashSet<String>>,
    #[serde(
        default,
        skip_serializing_if = "HashSet::is_empty",
        serialize_with = "serialize_sorted_hashset_of_strings"
    )]
    pub ignored_private_constants: HashSet<String>,
    #[serde(
        default,
        skip_serializing_if = "HashSet::is_empty",
        serialize_with = "serialize_sorted_hashset_of_strings"
    )]
    pub private_constants: HashSet<String>,
    #[serde(
        default = "default_public_folder",
        skip_serializing_if = "is_default_public_folder"
    )]
    pub public_folder: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer: Option<String>,
    #[serde(flatten)]
    pub client_keys: HashMap<String, Value>,
}

fn serialize_checker_setting<S>(
    value: &Option<String>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => {
            if value == "strict" {
                serializer.serialize_str("strict")
            } else {
                serializer.serialize_bool(value == "true")
            }
        }
        None => serializer.serialize_none(),
    }
}

fn is_default_public_folder(value: &String) -> bool {
    value == "app/public"
}

fn serialize_sorted_option_hashset_of_strings<S>(
    value: &Option<HashSet<String>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => serialize_sorted_hashset_of_strings(value, serializer),
        None => serializer.serialize_none(),
    }
}

fn serialize_sorted_hashset_of_strings<S>(
    value: &HashSet<String>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Serialize in sorted order
    let mut value: Vec<&String> = value.iter().collect();
    value.sort();
    value.serialize(serializer)
}

fn default_public_folder() -> String {
    "app/public".to_string()
}

pub(crate) fn from_path(package_yml_absolute_path: &Path) -> RawPack {
    let mut yaml_contents = String::new();
    let mut file = File::open(package_yml_absolute_path)
        .expect("Failed to open the YAML file");
    file.read_to_string(&mut yaml_contents)
        .expect("Failed to read the YAML file");

    let raw_pack: RawPack = serde_yaml::from_str(&yaml_contents)
        .expect("Failed to deserialize the YAML");

    raw_pack
}
