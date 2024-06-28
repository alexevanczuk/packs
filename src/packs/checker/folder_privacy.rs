use super::output_helper::print_reference_location;
use super::pack_checker::PackChecker;
use super::CheckerInterface;
use crate::packs::checker::reference::Reference;
use crate::packs::pack::Pack;
use crate::packs::{Configuration, Violation};

pub struct Checker {}

impl CheckerInterface for Checker {
    fn check(
        &self,
        reference: &Reference,
        configuration: &Configuration,
    ) -> anyhow::Result<Option<Violation>> {
        let pack_checker =
            PackChecker::new(configuration, reference, &self.violation_type())?;
        if !pack_checker.checkable()? {
            return Ok(None);
        }
        let defining_pack = pack_checker.defining_pack.unwrap();

        if !folder_visible(pack_checker.referencing_pack, defining_pack) {
            let loc = print_reference_location(reference);

            let message = format!(
                "{}Folder Privacy violation: `{}` belongs to `{}`, which is private to `{}` as it is not a sibling pack or parent pack.",
                loc,
                reference.constant_name,
                defining_pack.name,
                pack_checker.referencing_pack.name,
            );

            Ok(Some(Violation {
                message,
                identifier: pack_checker.violation_identifier(),
            }))
        } else {
            Ok(None)
        }
    }

    fn violation_type(&self) -> String {
        "folder_privacy".to_owned()
    }
}

fn folder_visible(referencing_pack: &Pack, defining_pack: &Pack) -> bool {
    if defining_pack.enforce_folder_privacy().is_false() {
        return true;
    }

    if referencing_pack.relative_path.to_string_lossy() == "." {
        return true; // root pack is visible to all
    }

    if let (Some(from_pack_parent_path), Some(to_pack_parent_path)) = (
        referencing_pack.relative_path.parent(),
        defining_pack.relative_path.parent(),
    ) {
        if from_pack_parent_path == to_pack_parent_path {
            return true; // siblings are visible to each other
        }
    }

    defining_pack
        .relative_path
        .to_string_lossy()
        .starts_with(referencing_pack.relative_path.to_string_lossy().as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs::{
        checker::common_test::tests::{
            build_expected_violation, default_defining_pack,
            default_referencing_pack, test_check, TestChecker,
        },
        pack::{CheckerSetting, EnforcementGlobsIgnore},
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
                enforce_folder_privacy: Some(CheckerSetting::True),
                ..default_defining_pack()
            }),
            referencing_pack: Pack{
                relative_path: PathBuf::from("packs/foo"),
                ..default_referencing_pack()},
            expected_violation: Some(build_expected_violation(
                "packs/foo/app/services/foo.rb:3:1\nFolder Privacy violation: `::Bar` belongs to `packs/bar`, which is private to `packs/foo` as it is not a sibling pack or parent pack.".to_string(),
                "folder_privacy".to_string(), false)),
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_with_enforcement_globs_ignore() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_folder_privacy: Some(CheckerSetting::True),
                enforcement_globs_ignore: Some(vec![EnforcementGlobsIgnore {
                    enforcements: ["folder_privacy"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    ignores: ["packs/foo/**"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    reason: "deprecated".to_string(),
                }]),
                ..default_defining_pack()
            }),
            referencing_pack: Pack {
                relative_path: PathBuf::from("packs/foo"),
                ..default_referencing_pack()
            },
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }
    #[test]
    fn test_with_strict_violation() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_folder_privacy: Some(CheckerSetting::Strict),
                ..default_defining_pack()
            }),
            referencing_pack: Pack{
                relative_path: PathBuf::from("packs/foo"),
                ..default_referencing_pack()},
            expected_violation: Some(build_expected_violation(
                "packs/foo/app/services/foo.rb:3:1\nFolder Privacy violation: `::Bar` belongs to `packs/bar`, which is private to `packs/foo` as it is not a sibling pack or parent pack.".to_string(),
                "folder_privacy".to_string(), true)),
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_no_violation() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_folder_privacy: Some(CheckerSetting::False),
                ..default_defining_pack()
            }),
            referencing_pack: Pack {
                relative_path: PathBuf::from("packs/foo"),
                ..default_referencing_pack()
            },
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    fn assert_folder_privacy(
        from_pack_path: &str,
        to_pack_path: &str,
        to_pack_enforce_folder_privacy: Option<CheckerSetting>,
        expected: bool,
    ) {
        let from_pack = Pack {
            name: from_pack_path.to_string(),
            relative_path: PathBuf::from(&from_pack_path),
            ..Pack::default()
        };
        if from_pack_path == to_pack_path {
            assert_eq!(expected, folder_visible(&from_pack, &from_pack));
            return;
        }
        let to_pack = Pack {
            name: to_pack_path.to_string(),
            relative_path: PathBuf::from(&to_pack_path),
            enforce_folder_privacy: to_pack_enforce_folder_privacy,
            ..Pack::default()
        };

        assert_eq!(expected, folder_visible(&from_pack, &to_pack));
    }

    #[test]
    fn test_folder_privacy_when_different_parent_invisible() {
        assert_folder_privacy(
            "packs/bars/bar",
            "packs/foos/foo",
            Some(CheckerSetting::True),
            false,
        );
    }

    #[test]
    fn test_folder_privacy_when_not_enforced() {
        assert_folder_privacy(
            "packs/bar",
            "packs/foos/zoo",
            Some(CheckerSetting::False),
            true,
        );
    }

    #[test]
    fn test_folder_privacy_when_siblings() {
        assert_folder_privacy(
            "packs/bar",
            "packs/foos",
            Some(CheckerSetting::True),
            true,
        );
    }

    #[test]
    fn test_folder_privacy_when_same() {
        assert_folder_privacy(
            "packs/bar",
            "packs/bar",
            Some(CheckerSetting::True),
            true,
        );
    }

    #[test]
    fn test_folder_privacy_when_descendant() {
        assert_folder_privacy(
            "packs/foo",
            "packs/foo/bar",
            Some(CheckerSetting::True),
            true,
        );
    }

    #[test]
    fn test_folder_privacy_when_parent_invisible() {
        assert_folder_privacy(
            "packs/foo/bar",
            "packs/foo",
            Some(CheckerSetting::True),
            false,
        );
    }

    #[test]
    fn test_folder_privacy_when_invisible() {
        assert_folder_privacy(
            "packs/baz",
            "packs/foos/foo",
            Some(CheckerSetting::True),
            false,
        );
    }

    #[test]
    fn test_folder_privacy_when_from_is_root() {
        assert_folder_privacy(
            ".",
            "packs/foos/foo",
            Some(CheckerSetting::True),
            true,
        );
    }
}
