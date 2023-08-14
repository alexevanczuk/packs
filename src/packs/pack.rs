use std::{
    collections::{HashMap, HashSet},
    fs::File,
    hash::Hasher,
    io::Read,
    path::{Path, PathBuf},
};

use core::hash::Hash;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_yaml::Value;

use super::{checker::ViolationIdentifier, PackageTodo};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Pack {
    #[serde(skip)]
    pub yml: PathBuf,

    #[serde(skip)]
    pub name: String,

    #[serde(skip)]
    pub relative_path: PathBuf,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_checker_setting",
        deserialize_with = "deserialize_checker_setting"
    )]
    pub enforce_dependencies: Option<CheckerSetting>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_checker_setting",
        deserialize_with = "deserialize_checker_setting"
    )]
    pub enforce_privacy: Option<CheckerSetting>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_checker_setting",
        deserialize_with = "deserialize_checker_setting"
    )]
    pub enforce_visibility: Option<CheckerSetting>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_checker_setting",
        deserialize_with = "deserialize_checker_setting"
    )]
    pub enforce_architecture: Option<CheckerSetting>,

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

    #[serde(skip)]
    pub package_todo: PackageTodo,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_sorted_option_hashset_of_strings"
    )]
    pub visible_to: Option<HashSet<String>>,

    #[serde(skip_serializing_if = "is_default_public_folder")]
    pub public_folder: Option<PathBuf>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer: Option<String>,

    #[serde(flatten)]
    pub client_keys: HashMap<String, Value>,
}

impl Hash for Pack {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Debug, Default, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub enum CheckerSetting {
    #[default]
    False,
    True,
    Strict,
}

impl CheckerSetting {
    pub fn is_false(&self) -> bool {
        matches!(self, Self::False)
    }

    pub fn is_strict(&self) -> bool {
        matches!(self, Self::Strict)
    }
}

impl Pack {
    pub fn all_violations(&self) -> Vec<ViolationIdentifier> {
        let mut violations = Vec::new();
        let violations_by_pack = &self.package_todo.violations_by_defining_pack;
        for (defining_pack_name, violation_groups) in violations_by_pack {
            for (constant_name, violation_group) in violation_groups {
                for violation_type in &violation_group.violation_types {
                    for file in &violation_group.files {
                        let identifier = ViolationIdentifier {
                            violation_type: violation_type.clone(),
                            file: file.clone(),
                            constant_name: constant_name.clone(),
                            referencing_pack_name: self.name.clone(),
                            defining_pack_name: defining_pack_name.clone(),
                        };

                        violations.push(identifier);
                    }
                }
            }
        }
        violations
    }

    pub fn from_path(
        package_yml_absolute_path: &Path,
        absolute_root: &Path,
    ) -> Pack {
        let mut yaml_contents = String::new();
        let mut file =
            File::open(package_yml_absolute_path).unwrap_or_else(|e| {
                panic!(
                    "Failed to open the YAML file at {:?} with error: {:?}",
                    package_yml_absolute_path, e
                )
            });

        file.read_to_string(&mut yaml_contents).unwrap_or_else(|e| {
            panic!(
                "Failed to read the YAML file at {:?} with error: {:?}",
                package_yml_absolute_path, e
            )
        });

        let absolute_path_to_package_todo = package_yml_absolute_path
            .parent()
            .unwrap()
            .join("package_todo.yml");

        let package_todo: PackageTodo = if absolute_path_to_package_todo
            .exists()
        {
            let mut package_todo_contents = String::new();
            let mut file = File::open(&absolute_path_to_package_todo)
                .expect("Failed to open the package_todo.yml file");
            file.read_to_string(&mut package_todo_contents)
                .expect("Could not read the package_todo.yml file");
            serde_yaml::from_str(&package_todo_contents).unwrap_or_else(|e| {

                panic!(
                    "Failed to deserialize the package_todo.yml file at {} with error {}",
                    absolute_path_to_package_todo.display(),
                    e
                )
            })
        } else {
            PackageTodo::default()
        };

        Pack::from_contents(
            package_yml_absolute_path,
            absolute_root,
            &yaml_contents,
            package_todo,
        )
    }

    pub fn from_contents(
        package_yml_absolute_path: &Path,
        absolute_root: &Path,
        package_yml_contents: &str,
        package_todo: PackageTodo,
    ) -> Pack {
        let pack: Pack = serde_yaml::from_str(package_yml_contents)
            .expect("Failed to deserialize the YAML");

        let package_yml_relative_path = package_yml_absolute_path
            .strip_prefix(absolute_root)
            .unwrap();
        let mut relative_path = package_yml_relative_path
            .parent()
            .expect("Expected package to be in a parent directory")
            .to_owned();

        let mut name = relative_path
            .to_str()
            .expect("Non-unicode characters?")
            .to_owned();
        let yml = package_yml_absolute_path;

        // Handle the root pack
        if name == *"" {
            name = String::from(".");
            relative_path = PathBuf::from(".");
        };

        let pack: Pack = Pack {
            yml: yml.to_path_buf(),
            name,
            relative_path,
            package_todo,
            ..pack
        };

        pack
    }

    pub fn relative_yml(&self) -> PathBuf {
        self.relative_path.join("package.yml")
    }

    pub(crate) fn enforce_architecture(&self) -> &CheckerSetting {
        match &self.enforce_architecture {
            Some(setting) => setting,
            None => &CheckerSetting::False,
        }
    }

    pub(crate) fn enforce_dependencies(&self) -> &CheckerSetting {
        match &self.enforce_dependencies {
            Some(setting) => setting,
            None => &CheckerSetting::False,
        }
    }

    pub(crate) fn enforce_privacy(&self) -> &CheckerSetting {
        match &self.enforce_privacy {
            Some(setting) => setting,
            None => &CheckerSetting::False,
        }
    }

    pub(crate) fn enforce_visibility(&self) -> &CheckerSetting {
        match &self.enforce_visibility {
            Some(setting) => setting,
            None => &CheckerSetting::False,
        }
    }

    pub(crate) fn public_folder(&self) -> PathBuf {
        match &self.public_folder {
            Some(folder) => folder.to_owned(),
            None => self.relative_path.join("app/public"),
        }
    }

    pub(crate) fn add_dependency(&self, to_pack: &Pack) -> Pack {
        let mut new_pack = self.clone();
        new_pack.dependencies.insert(to_pack.name.clone());
        new_pack
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

fn is_default_public_folder(value: &Option<PathBuf>) -> bool {
    match value {
        Some(value) => value == &PathBuf::from("app/public"),
        None => true,
    }
}

pub fn serialize_pack(pack: &Pack) -> String {
    let serialized_pack = serde_yaml::to_string(&pack).unwrap();
    // Indent dependencies by 2 spaces
    if serialized_pack == "{}\n" {
        "".to_owned()
    } else {
        serialized_pack.replace("\n-", "\n  -")
    }
}

pub fn write_pack_to_disk(pack: &Pack) {
    let serialized_pack = serialize_pack(pack);
    let pack_dir = pack.yml.parent().unwrap_or_else(|| {
        panic!("Failed to get parent directory of pack {:?}", &pack.yml)
    });

    std::fs::create_dir_all(pack_dir).unwrap_or_else(|e| {
        panic!(
            "Failed to create directory for pack {:?} with error {:?}",
            &pack_dir, e
        )
    });
    std::fs::write(&pack.yml, serialized_pack).unwrap_or_else(|e| {
        panic!(
            "Failed to write pack to disk {:?} with error {:?}",
            &pack.yml, e
        )
    });
}

fn serialize_checker_setting<S>(
    value: &Option<CheckerSetting>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(CheckerSetting::False) => serializer.serialize_bool(false),
        Some(CheckerSetting::True) => serializer.serialize_bool(true),
        Some(CheckerSetting::Strict) => serializer.serialize_str("strict"),
        None => serializer.serialize_none(),
    }
}

fn deserialize_checker_setting<'de, D>(
    deserializer: D,
) -> Result<Option<CheckerSetting>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize an optional String
    let s = String::deserialize(deserializer);

    match s.unwrap().as_str() {
        "false" => Ok(Some(CheckerSetting::False)),
        "true" => Ok(Some(CheckerSetting::True)),
        "strict" => Ok(Some(CheckerSetting::Strict)),
        _ => Err(serde::de::Error::custom(
            "expected one of: false, true, strict",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn reserialize_pack(pack_yml: &str) -> String {
        let deserialized_pack = serde_yaml::from_str::<Pack>(pack_yml).unwrap();
        serialize_pack(&deserialized_pack)
    }

    #[test]
    fn test_serde_sorted_dependencies() {
        let pack_yml = r#"
# some comment
dependencies:
  - packs/c
  - packs/a
  - packs/b
"#;

        let actual = reserialize_pack(pack_yml);

        let expected = r#"
dependencies:
  - packs/a
  - packs/b
  - packs/c
"#
        .trim_start();

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_serde_with_enforcements() {
        let pack_yml = r#"
# some comment
enforce_privacy: true
enforce_dependencies: strict
dependencies:
  - packs/c
  - packs/a
  - packs/b
foobar: true
"#;

        let actual = reserialize_pack(pack_yml);

        let expected = r#"
enforce_dependencies: strict
enforce_privacy: true
dependencies:
  - packs/a
  - packs/b
  - packs/c
foobar: true
"#
        .trim_start();

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_serde_with_arbitrary_client_keys() {
        let pack_yml = r#"
# some comment
dependencies:
  - packs/c
  - packs/a
  - packs/b
foobar: true
"#;

        let actual = reserialize_pack(pack_yml);

        let expected = r#"
dependencies:
  - packs/a
  - packs/b
  - packs/c
foobar: true
"#
        .trim_start();

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_serde_with_explicitly_empty_visible() {
        let pack_yml = r#"
visible_to:
  - packs/c
  - packs/a
  - packs/b
"#;

        let actual = reserialize_pack(pack_yml);

        let expected = r#"
visible_to:
  - packs/a
  - packs/b
  - packs/c
"#
        .trim_start();

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_serde_with_metadata() {
        let pack_yml = r#"
enforce_dependencies: false
metadata:
  foobar: true
"#;

        let actual = reserialize_pack(pack_yml);

        let expected = r#"
enforce_dependencies: false
metadata:
  foobar: true
"#
        .trim_start();

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_serde_with_owner() {
        let pack_yml = r#"
owner: Foobar
enforce_dependencies: true
"#;

        let actual = reserialize_pack(pack_yml);

        let expected = r#"
enforce_dependencies: true
owner: Foobar
"#
        .trim_start();

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_serde_with_empty_pack() {
        let pack_yml = r#""#;

        let actual = reserialize_pack(pack_yml);

        let expected = r#""#.trim_start();

        assert_eq!(expected, actual)
    }
}
