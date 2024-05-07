use super::{CheckerInterface, ViolationIdentifier};
use crate::packs::checker::reference::Reference;
use crate::packs::pack::Pack;
use crate::packs::{Configuration, Violation};
use anyhow::Result;

pub struct Checker {}

impl CheckerInterface for Checker {
    fn check(
        &self,
        reference: &Reference,
        configuration: &Configuration,
    ) -> anyhow::Result<Option<Violation>> {
        let pack_set = &configuration.pack_set;
        let referencing_pack = &reference.referencing_pack(pack_set)?;
        let relative_defining_file = &reference.relative_defining_file;
        if relative_defining_file.is_none() {
            return Ok(None);
        }
        let defining_pack = &reference.defining_pack(pack_set)?;
        if defining_pack.is_none() {
            return Ok(None);
        }
        let defining_pack = defining_pack.unwrap();
        if !folder_visible(referencing_pack, defining_pack).unwrap() {
            let message = format!(
                "{}:{}:{}\nFolder Visibility violation: `{}` belongs to `{}`, which is not visible to `{}` as it is not a sibling pack or parent pack.",
                reference.relative_referencing_file,
                reference.source_location.line,
                reference.source_location.column,
                reference.constant_name,
                defining_pack.name,
                referencing_pack.name,
            );
            let identifier = ViolationIdentifier {
                violation_type: self.violation_type(),
                strict: defining_pack.enforce_folder_visibility().is_strict(),
                file: reference.relative_referencing_file.clone(),
                constant_name: reference.constant_name.clone(),
                referencing_pack_name: referencing_pack.name.clone(),
                defining_pack_name: defining_pack.name.clone(),
            };
            Ok(Some(Violation {
                message,
                identifier,
            }))
        } else {
            Ok(None)
        }
    }

    fn violation_type(&self) -> String {
        "folder_visibility".to_owned()
    }
}

fn folder_visible(from_pack: &Pack, to_pack: &Pack) -> Result<bool> {
    if to_pack.enforce_folder_visibility().is_false() {
        return Ok(true);
    }

    if from_pack.relative_path.to_string_lossy() == "." {
        return Ok(true); // root pack is visible to all
    }

    if let (Some(from_pack_parent_path), Some(to_pack_parent_path)) = (
        from_pack.relative_path.parent(),
        to_pack.relative_path.parent(),
    ) {
        if from_pack_parent_path == to_pack_parent_path {
            return Ok(true); // siblings are visible to each other
        }
    }
    // visible if "to" is a descendant of "from"
    Ok(to_pack
        .relative_path
        .to_string_lossy()
        .starts_with(from_pack.relative_path.to_string_lossy().as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs::{
        checker::common_test::tests::{
            build_expected_violation, default_defining_pack,
            default_referencing_pack, test_check, TestChecker,
        },
        pack::CheckerSetting,
    };
    use std::path::PathBuf;

    #[test]
    fn test_with_violation() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_folder_visibility: Some(CheckerSetting::True),
                ..default_defining_pack()
            }),
            referencing_pack: Pack{
                relative_path: PathBuf::from("packs/foo"),
                ..default_referencing_pack()},
            expected_violation: Some(build_expected_violation(
                "packs/foo/app/services/foo.rb:3:1\nFolder Visibility violation: `::Bar` belongs to `packs/bar`, which is not visible to `packs/foo` as it is not a sibling pack or parent pack.".to_string(),
                "folder_visibility".to_string(), false)),
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker, vec![])
    }
    #[test]
    fn test_with_strict_violation() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_folder_visibility: Some(CheckerSetting::Strict),
                ..default_defining_pack()
            }),
            referencing_pack: Pack{
                relative_path: PathBuf::from("packs/foo"),
                ..default_referencing_pack()},
            expected_violation: Some(build_expected_violation(
                "packs/foo/app/services/foo.rb:3:1\nFolder Visibility violation: `::Bar` belongs to `packs/bar`, which is not visible to `packs/foo` as it is not a sibling pack or parent pack.".to_string(),
                "folder_visibility".to_string(), true)),
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker, vec![])
    }

    #[test]
    fn test_no_violation() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_folder_visibility: Some(CheckerSetting::False),
                ..default_defining_pack()
            }),
            referencing_pack: Pack {
                relative_path: PathBuf::from("packs/foo"),
                ..default_referencing_pack()
            },
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker, vec![])
    }

    fn assert_folder_visibility(
        from_pack_path: &str,
        to_pack_path: &str,
        to_pack_enforce_folder_visibility: Option<CheckerSetting>,
        expected: bool,
    ) {
        let from_pack = Pack {
            name: from_pack_path.to_string(),
            relative_path: PathBuf::from(&from_pack_path),
            ..Pack::default()
        };
        if from_pack_path == to_pack_path {
            assert_eq!(
                expected,
                folder_visible(&from_pack, &from_pack).unwrap()
            );
            return;
        }
        let to_pack = Pack {
            name: to_pack_path.to_string(),
            relative_path: PathBuf::from(&to_pack_path),
            enforce_folder_visibility: to_pack_enforce_folder_visibility,
            ..Pack::default()
        };

        assert_eq!(expected, folder_visible(&from_pack, &to_pack).unwrap());
    }

    #[test]
    fn test_folder_visibility_when_different_parent_invisible() {
        assert_folder_visibility(
            "packs/bars/bar",
            "packs/foos/foo",
            Some(CheckerSetting::True),
            false,
        );
    }

    #[test]
    fn test_folder_visibility_when_not_enforced() {
        assert_folder_visibility(
            "packs/bar",
            "packs/foos/zoo",
            Some(CheckerSetting::False),
            true,
        );
    }

    #[test]
    fn test_folder_visibility_when_siblings() {
        assert_folder_visibility(
            "packs/bar",
            "packs/foos",
            Some(CheckerSetting::True),
            true,
        );
    }

    #[test]
    fn test_folder_visibility_when_same() {
        assert_folder_visibility(
            "packs/bar",
            "packs/bar",
            Some(CheckerSetting::True),
            true,
        );
    }

    #[test]
    fn test_folder_visibility_when_descendant() {
        assert_folder_visibility(
            "packs/foo",
            "packs/foo/bar",
            Some(CheckerSetting::True),
            true,
        );
    }

    #[test]
    fn test_folder_visibility_when_parent_invisible() {
        assert_folder_visibility(
            "packs/foo/bar",
            "packs/foo",
            Some(CheckerSetting::True),
            false,
        );
    }

    #[test]
    fn test_folder_visibility_when_invisible() {
        assert_folder_visibility(
            "packs/baz",
            "packs/foos/foo",
            Some(CheckerSetting::True),
            false,
        );
    }

    #[test]
    fn test_folder_visibility_when_from_is_root() {
        assert_folder_visibility(
            ".",
            "packs/foos/foo",
            Some(CheckerSetting::True),
            true,
        );
    }
}
