use std::collections::HashSet;

use super::{CheckerInterface, ViolationIdentifier};
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
        let referencing_pack =
            &reference.referencing_pack(&configuration.pack_set)?;
        let relative_defining_file = &reference.relative_defining_file;

        let referencing_pack_name = &referencing_pack.name;
        let defining_pack =
            &reference.defining_pack(&configuration.pack_set)?;
        if defining_pack.is_none() {
            return Ok(None);
        }
        let defining_pack = defining_pack.unwrap();

        if defining_pack.enforce_visibility().is_false() {
            return Ok(None);
        }

        if defining_pack
            .visible_to
            .as_ref()
            .unwrap_or(&HashSet::new())
            .contains(referencing_pack_name)
        {
            return Ok(None);
        }

        let defining_pack_name = &defining_pack.name;

        if relative_defining_file.is_none() {
            return Ok(None);
        }

        if referencing_pack_name == defining_pack_name {
            return Ok(None);
        }

        if defining_pack.is_ignored(
            &reference.relative_referencing_file,
            &self.violation_type(),
        )? {
            return Ok(None);
        }

        let message = format!(
            "\x1b[34m{}\x1b[0m:{}:{}\nVisibility violation: `{}` belongs to `{}`, which is not visible to `{}`",
            reference.relative_referencing_file,
            reference.source_location.line,
            reference.source_location.column,
            reference.constant_name,
            defining_pack_name,
            referencing_pack_name,
        );

        let violation_type = String::from("visibility");
        let file = reference.relative_referencing_file.clone();
        let identifier = ViolationIdentifier {
            violation_type,
            strict: defining_pack.enforce_visibility().is_strict(),
            file,
            constant_name: reference.constant_name.clone(),
            referencing_pack_name: referencing_pack_name.clone(),
            defining_pack_name: defining_pack_name.clone(),
        };

        Ok(Some(Violation {
            message,
            identifier,
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
