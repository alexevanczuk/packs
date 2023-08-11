use std::{collections::HashSet, fs};

use itertools::Itertools;

use super::{
    raw_pack::{self, RawPack},
    Configuration,
};

pub(crate) fn lint_package_yml_files(configuration: &Configuration) {
    for pack in &configuration.pack_set.packs {
        let pack_yml = pack.yml.clone();
        let raw_pack = raw_pack::from_path(&pack_yml);

        let linted_pack_yml = linted_package_yml(&raw_pack);

        // Write the linted YAML content back to the file
        fs::write(&pack_yml, linted_pack_yml)
            .expect("Failed to write linted YAML to file");
    }
}

pub(crate) fn linted_package_yml(raw_pack: &RawPack) -> String {
    let sorted_dependencies = raw_pack.dependencies.clone();
    let sorted_dependencies: HashSet<String> = sorted_dependencies
        .iter()
        .map(|dependency| dependency.to_string())
        .sorted_by(|a, b| a.cmp(b))
        .collect();

    let pack_with_sorted_dependencies = RawPack {
        dependencies: sorted_dependencies,
        ..raw_pack.clone()
    };

    serde_yaml::to_string(&pack_with_sorted_dependencies)
        .unwrap()
        // Indent dependencies by 2 spaces
        .replace("\n-", "\n  -")
}

#[cfg(test)]
mod tests {
    use crate::packs::raw_pack::RawPack;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_linted_package_yml_with_dependencies() {
        let pack_yml = r#"
# some comment
dependencies:
  - packs/c
  - packs/a
  - packs/b
"#;

        let pack = serde_yaml::from_str::<RawPack>(pack_yml).unwrap();

        let expected_linted_pack_yml = r#"
dependencies:
  - packs/a
  - packs/b
  - packs/c
"#;

        assert_eq!(
            expected_linted_pack_yml.trim_start(),
            linted_package_yml(&pack)
        )
    }

    #[test]
    fn test_linted_package_yml_with_enforcements() {
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

        let pack = serde_yaml::from_str::<RawPack>(pack_yml).unwrap();

        let expected_linted_pack_yml = r#"
enforce_dependencies: strict
enforce_privacy: true
dependencies:
  - packs/a
  - packs/b
  - packs/c
foobar: true
"#;

        assert_eq!(
            expected_linted_pack_yml.trim_start(),
            linted_package_yml(&pack)
        )
    }

    #[test]
    fn test_linted_package_yml_with_user_keys() {
        let pack_yml = r#"
# some comment
dependencies:
  - packs/c
  - packs/a
  - packs/b
foobar: true
"#;

        let pack = serde_yaml::from_str::<RawPack>(pack_yml).unwrap();

        let expected_linted_pack_yml = r#"
dependencies:
  - packs/a
  - packs/b
  - packs/c
foobar: true
"#;

        assert_eq!(
            expected_linted_pack_yml.trim_start(),
            linted_package_yml(&pack)
        )
    }

    #[test]
    fn test_linted_package_yml_with_explicitly_empty_visible() {
        let pack_yml = r#"
visible_to:
  - packs/c
  - packs/a
  - packs/b
"#;

        let pack = serde_yaml::from_str::<RawPack>(pack_yml).unwrap();

        let expected_linted_pack_yml = r#"
visible_to:
  - packs/a
  - packs/b
  - packs/c
"#;

        assert_eq!(
            expected_linted_pack_yml.trim_start(),
            linted_package_yml(&pack)
        )
    }

    #[test]
    fn test_linted_package_yml_with_metadata() {
        let pack_yml = r#"
enforce_dependencies: false
metadata:
  foobar: true
"#;

        let pack = serde_yaml::from_str::<RawPack>(pack_yml).unwrap();

        let expected_linted_pack_yml = r#"
enforce_dependencies: false
metadata:
  foobar: true
"#;

        assert_eq!(
            expected_linted_pack_yml.trim_start(),
            linted_package_yml(&pack)
        )
    }
}
