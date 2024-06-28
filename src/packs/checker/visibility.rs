use std::collections::HashSet;

use super::output_helper::print_reference_location;
use super::pack_checker::PackChecker;
use super::CheckerInterface;
use crate::packs::checker::Reference;
use crate::packs::{Configuration, Violation};

pub struct Checker {}

// TODO:
// Once we implement packs validate, we need to ensure that nothing can add a dependency
// from a pack to a pack that doesn't permit visibility from the referencing pack
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
        if defining_pack
            .visible_to
            .as_ref()
            .unwrap_or(&HashSet::new())
            .contains(&pack_checker.referencing_pack.name)
        {
            return Ok(None);
        }

        let loc = print_reference_location(reference);

        let message = format!(
            "{}Visibility violation: `{}` belongs to `{}`, which is not visible to `{}`",
            loc,
            reference.constant_name,
            defining_pack.name,
            pack_checker.referencing_pack.name,
        );

        Ok(Some(Violation {
            message,
            identifier: pack_checker.violation_identifier(),
        }))
    }

    fn violation_type(&self) -> String {
        "visibility".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use self::packs::{
        checker::common_test::tests::{
            build_expected_violation, default_defining_pack,
            default_referencing_pack, test_check, TestChecker,
        },
        pack::EnforcementGlobsIgnore,
    };

    use super::*;
    use crate::packs::{
        pack::{CheckerSetting, Pack},
        *,
    };

    #[test]
    fn referencing_and_defining_pack_are_identical() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_visibility: Some(CheckerSetting::True),
                ..default_defining_pack()
            }),
            referencing_pack: Pack {
                name: "packs/bar".to_owned(),
                relative_path: PathBuf::from("packs/bar"),
                ..default_referencing_pack()
            },
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_with_violation() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_visibility: Some(CheckerSetting::True),
                ..default_defining_pack()
            }),
            referencing_pack: Pack{
                relative_path: PathBuf::from("packs/foo"),
                ..default_referencing_pack()},
            expected_violation: Some(build_expected_violation(
                "packs/foo/app/services/foo.rb:3:1\nVisibility violation: `::Bar` belongs to `packs/bar`, which is not visible to `packs/foo`".to_string(),
                "visibility".to_string(), false)),
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
                enforce_visibility: Some(CheckerSetting::True),
                enforcement_globs_ignore: Some(vec![EnforcementGlobsIgnore {
                    enforcements: ["visibility"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    ignores: ["packs/foo/**"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    reason: "foo is deprecated".to_string(),
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
                enforce_visibility: Some(CheckerSetting::Strict),
                ..default_defining_pack()
            }),
            referencing_pack: Pack{
                relative_path: PathBuf::from("packs/foo"),
                ..default_referencing_pack()},
            expected_violation: Some(build_expected_violation(
                "packs/foo/app/services/foo.rb:3:1\nVisibility violation: `::Bar` belongs to `packs/bar`, which is not visible to `packs/foo`".to_string(),
                "visibility".to_string(), true)),
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn reference_is_not_a_visibility_violation() -> anyhow::Result<()> {
        let mut visible_to = HashSet::new();
        visible_to.insert(String::from("packs/foo"));

        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_visibility: Some(CheckerSetting::True),
                visible_to: Some(visible_to),
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
}
