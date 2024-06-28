use std::{
    collections::{HashMap, HashSet},
    fs::File,
    hash::Hasher,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::Context;
use core::hash::Hash;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_yaml::Value;

use super::{
    checker::ViolationIdentifier, file_utils::expand_glob, ignored, PackageTodo,
};

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
    pub enforce_layers: Option<CheckerSetting>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer: Option<String>,

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

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_checker_setting",
        deserialize_with = "deserialize_checker_setting"
    )]
    pub enforce_folder_privacy: Option<CheckerSetting>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_checker_setting",
        deserialize_with = "deserialize_checker_setting"
    )]
    pub enforce_folder_visibility: Option<CheckerSetting>, // deprecated

    #[serde(skip_serializing_if = "is_default_public_folder")]
    pub public_folder: Option<PathBuf>,

    #[serde(flatten)]
    pub client_keys: HashMap<String, Value>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enforcement_globs_ignore: Option<Vec<EnforcementGlobsIgnore>>,
}

impl Hash for Pack {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Debug, Default, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct EnforcementGlobsIgnore {
    #[serde(
        default,
        serialize_with = "serialize_sorted_hashset_of_strings",
        skip_serializing_if = "HashSet::is_empty"
    )]
    pub enforcements: HashSet<String>,

    #[serde(
        default,
        serialize_with = "serialize_sorted_hashset_of_strings",
        skip_serializing_if = "HashSet::is_empty"
    )]
    pub ignores: HashSet<String>,

    #[serde(default)]
    pub reason: String,
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
                            strict: false,
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
    ) -> anyhow::Result<Pack> {
        let mut yaml_contents = String::new();

        let mut file = File::open(package_yml_absolute_path).map_err(|e| {
            anyhow::Error::new(e).context(format!(
                "Failed to open the YAML file at {:?}",
                package_yml_absolute_path
            ))
        })?;

        file.read_to_string(&mut yaml_contents).map_err(|e| {
            anyhow::Error::new(e).context(format!(
                "Failed to read the YAML file at {:?}",
                package_yml_absolute_path
            ))
        })?;

        let absolute_path_to_package_todo = package_yml_absolute_path
            .parent()
            .unwrap()
            .join("package_todo.yml");

        let package_todo: PackageTodo = if absolute_path_to_package_todo
            .exists()
        {
            let mut package_todo_contents = String::new();
            let mut file = File::open(&absolute_path_to_package_todo)
                .context("Failed to open the package_todo.yml file")?;
            file.read_to_string(&mut package_todo_contents)
                .context("Could not read the package_todo.yml file")?;
            serde_yaml::from_str(&package_todo_contents).with_context(|| {
                format!(
                    "Failed to deserialize the package_todo.yml file at {}. Try deleting the file and running the `update` command to regenerate it.",
                    absolute_path_to_package_todo.display()
                )
            })?
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
    ) -> anyhow::Result<Pack> {
        let pack_result = serde_yaml::from_str(package_yml_contents);
        let pack = match pack_result {
            Ok(pack) => pack,
            Err(e) => {
                anyhow::bail!(
                    "Failed to deserialize the YAML at {:?} with error: {:?}",
                    package_yml_absolute_path,
                    e
                )
            }
        };

        let package_yml_relative_path = package_yml_absolute_path
            .strip_prefix(absolute_root)
            .unwrap();
        let mut relative_path = package_yml_relative_path
            .parent()
            .context("Expected package to be in a parent directory")?
            .to_owned();

        let mut name = relative_path
            .to_str()
            .context("Non-unicode characters?")?
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

        Ok(pack)
    }

    pub fn default_autoload_roots(&self) -> Vec<PathBuf> {
        let root_pattern = self.yml.parent().unwrap().join("app").join("*");
        let concerns_pattern = root_pattern.join("concerns");
        let mut roots = expand_glob(root_pattern.to_str().unwrap());
        roots.extend(expand_glob(concerns_pattern.to_str().unwrap()));

        roots
    }

    pub fn relative_yml(&self) -> PathBuf {
        self.relative_path.join("package.yml")
    }

    pub(crate) fn enforce_folder_privacy(&self) -> &CheckerSetting {
        if self.enforce_folder_privacy.is_none() {
            // enforce_folder_visibility is deprecated
            match &self.enforce_folder_visibility {
                Some(setting) => setting,
                None => &CheckerSetting::False,
            }
        } else {
            match &self.enforce_folder_privacy {
                Some(setting) => setting,
                None => &CheckerSetting::False,
            }
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

    pub(crate) fn ignores_for_enforcement(
        &self,
        enforcement: &str,
    ) -> Option<&HashSet<String>> {
        self.enforcement_globs_ignore.as_ref().and_then(|ignores| {
            ignores
                .iter()
                .find(|ignore| ignore.enforcements.contains(enforcement))
                .map(|ignore| &ignore.ignores)
        })
    }

    pub(crate) fn is_ignored(
        &self,
        file_path: &str,
        enforcement: &str,
    ) -> anyhow::Result<bool> {
        if let Some(ignore_rules) = self.ignores_for_enforcement(enforcement) {
            return ignored::is_ignored(ignore_rules, file_path);
        }
        Ok(false)
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
    if serialized_pack == "{}\n" {
        "".to_owned()
    } else {
        serialized_pack
    }
}

pub fn write_pack_to_disk(pack: &Pack) -> anyhow::Result<()> {
    let serialized_pack = serialize_pack(pack);
    let pack_dir = pack.yml.parent().ok_or_else(|| {
        anyhow::Error::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Failed to get parent directory of pack {:?}", &pack.yml),
        ))
    })?;

    std::fs::create_dir_all(pack_dir).map_err(|e| {
        anyhow::Error::new(e).context(format!(
            "Failed to create directory for pack {:?}",
            &pack_dir
        ))
    })?;

    std::fs::write(&pack.yml, serialized_pack).map_err(|e| {
        anyhow::Error::new(e)
            .context(format!("Failed to write pack to disk {:?}", &pack.yml))
    })?;

    Ok(())
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
    use crate::test_util;

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
    fn test_serde_with_duplicate_dependencies() {
        let pack_yml = r#"
dependencies:
  - packs/a
  - packs/b
  - packs/a
  - packs/a
  - packs/a
"#;

        let actual = reserialize_pack(pack_yml);

        let expected = r#"
dependencies:
- packs/a
- packs/b
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
    fn test_serde_with_enforcement_globs() {
        let pack_yml = r#"
enforcement_globs_ignore:
  - enforcements:
      - privacy
    ignores:
      - "**/*"
      - "!packs/foo"
    reason: "deprecated foo"
  - enforcements:
      - layer
    ignores:
      - packs/bar
    reason: "deprecated bar"
        "#
        .trim_start();

        let pack: Result<Pack, _> = serde_yaml::from_str(pack_yml);
        let pack = pack.unwrap();
        assert_eq!(
            pack.clone().enforcement_globs_ignore.unwrap(),
            vec![
                EnforcementGlobsIgnore {
                    enforcements: ["privacy"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    ignores: ["**/*", "!packs/foo"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    reason: "deprecated foo".to_string(),
                },
                EnforcementGlobsIgnore {
                    enforcements: ["layer"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    ignores: ["packs/bar"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    reason: "deprecated bar".to_string(),
                },
            ]
        );

        let reserialized = reserialize_pack(pack_yml);
        let re_pack: Result<Pack, _> = serde_yaml::from_str(&reserialized);
        let re_pack = re_pack.unwrap();
        assert_eq!(pack, re_pack);

        assert_eq!(
            pack.ignores_for_enforcement("privacy"),
            Some(&{
                ["**/*", "!packs/foo"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            })
        );
        assert_eq!(pack.ignores_for_enforcement("nope"), None);
    }

    #[test]
    fn test_serde_with_empty_pack() {
        let pack_yml = r#""#;

        let actual = reserialize_pack(pack_yml);

        let expected = r#""#.trim_start();

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_autoload_roots() {
        let root = test_util::get_absolute_root(test_util::SIMPLE_APP);
        let pack =
            Pack::from_path(root.join("package.yml").as_path(), root.as_path());
        assert!(pack.is_ok());

        let actual = pack.unwrap().default_autoload_roots();
        let expected =
            vec![root.join("app/company_data"), root.join("app/services")];
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_all_recorded_violations() -> anyhow::Result<()> {
        let root = test_util::get_absolute_root(
            "tests/fixtures/contains_package_todo",
        );
        let pack = Pack::from_path(
            root.join("packs/foo/package.yml").as_path(),
            root.as_path(),
        )?;

        let mut actual = pack.all_violations();
        actual.sort_by(|a, b| a.file.cmp(&b.file));

        let expected = vec![
            ViolationIdentifier {
                violation_type: "dependency".to_string(),
                strict: false,
                file: "packs/foo/app/services/foo.rb".to_string(),
                constant_name: "::Bar".to_string(),
                referencing_pack_name: "packs/foo".to_string(),
                defining_pack_name: "packs/bar".to_string(),
            },
            ViolationIdentifier {
                violation_type: "dependency".to_string(),
                strict: false,
                file: "packs/foo/app/services/other_foo.rb".to_string(),
                constant_name: "::Bar".to_string(),
                referencing_pack_name: "packs/foo".to_string(),
                defining_pack_name: "packs/bar".to_string(),
            },
        ];

        assert_eq!(expected, actual);

        Ok(())
    }
}
