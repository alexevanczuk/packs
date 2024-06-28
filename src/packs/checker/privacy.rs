use super::output_helper::print_reference_location;
use super::pack_checker::PackChecker;
use super::CheckerInterface;
use crate::packs::checker::Reference;
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
        if defining_pack
            .ignored_private_constants
            .contains(&reference.constant_name)
        {
            return Ok(None);
        }

        // This is a hack for now – we need to read package.yml file public_paths at some point,
        // and probably find a better way to check if the constant is public

        let public_folder = &defining_pack.public_folder();
        let is_public = reference
            .relative_defining_file
            .as_ref()
            .unwrap()
            .starts_with(public_folder.to_string_lossy().as_ref());

        // Note this means that if the constant is ALSO in the list of private_constants,
        // it will be considered public.
        // This is how packwerk does it today.
        // Later we might want to add some sort of validation that a constant can be in the public folder OR in the list of private_constants,
        // but not both.
        if is_public {
            return Ok(None);
        }

        let private_constants = &defining_pack.private_constants;

        if !private_constants.is_empty() {
            let constant_is_private =
                private_constants.contains(&reference.constant_name);

            let constant_is_in_private_namespace =
                private_constants.iter().any(|private_constant| {
                    let namespaced_constant =
                        &format!("{}::", private_constant);
                    reference.constant_name.starts_with(namespaced_constant)
                });
            dbg!(constant_is_private, constant_is_in_private_namespace);
            if !constant_is_private && !constant_is_in_private_namespace {
                return Ok(None);
            }
        }

        // START: Original packwerk message
        // path/to/file.rb:36:0
        // Privacy violation: '::Constant' is private to 'packs/defining_pack' but referenced from 'packs/referencing_pack'.
        // Is there a public entrypoint in 'packs/defining_pack/app/public/' that you can use instead?

        // Inference details: this is a reference to ::Constant which seems to be defined in packs/defining_pack/path/to/definition.rb.
        // To receive help interpreting or resolving this error message, see: https://github.com/Shopify/packwerk/blob/main/TROUBLESHOOT.md#Troubleshooting-violations
        // END: Original packwerk message
        let loc = print_reference_location(reference);

        let message = format!(
            "{}Privacy violation: `{}` is private to `{}`, but referenced from `{}`",
            loc,
            reference.constant_name,
            defining_pack.name,
            &pack_checker.referencing_pack.name,
        );

        Ok(Some(Violation {
            message,
            identifier: pack_checker.violation_identifier(),
        }))
    }

    fn violation_type(&self) -> String {
        "privacy".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use self::packs::{
        checker::common_test::tests::{
            build_expected_violation, build_expected_violation_with_constant,
            default_defining_pack, default_referencing_pack, test_check,
            TestChecker,
        },
        pack::EnforcementGlobsIgnore,
    };

    use super::*;
    use crate::packs::{
        pack::{CheckerSetting, Pack},
        *,
    };

    #[test]
    fn test_reference_and_defining_packs_are_identical() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/foo".to_owned(),
                enforce_privacy: Some(CheckerSetting::True),
                ignored_private_constants: HashSet::from([String::from(
                    "::Bar",
                )]),
                ..default_defining_pack()
            }),
            referencing_pack: default_referencing_pack(),
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_with_ignored_private_constants() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_privacy: Some(CheckerSetting::True),
                ignored_private_constants: HashSet::from([String::from(
                    "::Bar",
                )]),
                ..default_defining_pack()
            }),
            referencing_pack: default_referencing_pack(),
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_with_privacy_violation() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_privacy: Some(CheckerSetting::True),
                ignored_private_constants: HashSet::from([String::from("::Taco")]),
                ..default_defining_pack()
            }),
            referencing_pack: default_referencing_pack(),
            expected_violation: Some(build_expected_violation(
                String::from("packs/foo/app/services/foo.rb:3:1\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"),
                String::from("privacy"), false,
            )),
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_with_strict_privacy_violation() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_privacy: Some(CheckerSetting::Strict),
                ignored_private_constants: HashSet::from([String::from("::Taco")]),
                ..default_defining_pack()
            }),
            referencing_pack: default_referencing_pack(),
            expected_violation: Some(build_expected_violation(
                String::from("packs/foo/app/services/foo.rb:3:1\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"),
                String::from("privacy"), true,
            )),
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_without_privacy_enforcement() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_privacy: Some(CheckerSetting::False),
                ignored_private_constants: HashSet::from([String::from(
                    "::Taco",
                )]),
                ..default_defining_pack()
            }),
            referencing_pack: default_referencing_pack(),
            ..Default::default()
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
                enforce_privacy: Some(CheckerSetting::True),
                ignored_private_constants: HashSet::from([String::from(
                    "::Taco",
                )]),
                enforcement_globs_ignore: Some(vec![EnforcementGlobsIgnore {
                    enforcements: ["privacy"]
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
            referencing_pack: default_referencing_pack(),
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_with_public_constant() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: Some(Reference {
                constant_name: String::from("::Bar"),
                defining_pack_name: Some(String::from("packs/bar")),
                referencing_pack_name: String::from("packs/foo"),
                relative_referencing_file: String::from(
                    "packs/foo/app/services/foo.rb",
                ),
                relative_defining_file: Some(String::from(
                    "packs/bar/app/public/bar.rb",
                )),
                source_location: SourceLocation { line: 3, column: 1 },
            }),
            configuration: None,
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_privacy: Some(CheckerSetting::True),
                public_folder: Some(PathBuf::from("packs/bar/app/public")),
                ..default_defining_pack()
            }),
            referencing_pack: default_referencing_pack(),
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_public_folder_detection() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: Some(Reference {
                constant_name: String::from("::Bar"),
                defining_pack_name: Some(String::from("packs/bar")),
                referencing_pack_name: String::from("packs/foo"),
                relative_referencing_file: String::from(
                    "packs/foo/app/services/foo.rb",
                ),
                relative_defining_file: Some(String::from(
                    "packs/bar/app/public/bar.rb",
                )),
                source_location: SourceLocation { line: 3, column: 1 },
            }),
            configuration: None,
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_privacy: Some(CheckerSetting::True),
                relative_path: PathBuf::from("packs/bar"),
                ..default_defining_pack()
            }),
            referencing_pack: default_referencing_pack(),
            expected_violation: None,
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_custom_public_folder_detection() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: Some(Reference {
                constant_name: String::from("::Bar"),
                defining_pack_name: Some(String::from("packs/bar")),
                referencing_pack_name: String::from("packs/foo"),
                relative_referencing_file: String::from(
                    "packs/foo/app/services/foo.rb",
                ),
                relative_defining_file: Some(String::from(
                    "packs/bar/app/api/bar.rb",
                )),
                source_location: SourceLocation { line: 3, column: 1 },
            }),
            configuration: None,
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_privacy: Some(CheckerSetting::True),
                public_folder: Some(PathBuf::from("packs/bar/app/api")),
                ..default_defining_pack()
            }),
            referencing_pack: default_referencing_pack(),
            expected_violation: None,
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_privacy_constants_exclude_referenced_constant() -> anyhow::Result<()>
    {
        let mut test_checker = TestChecker {
            reference: Some(Reference {
                constant_name: String::from("::Different"),
                defining_pack_name: Some(String::from("packs/bar")),
                referencing_pack_name: String::from("packs/foo"),
                relative_referencing_file: String::from(
                    "packs/foo/app/services/foo.rb",
                ),
                relative_defining_file: Some(String::from(
                    "packs/bar/app/services/bar.rb",
                )),
                source_location: SourceLocation { line: 3, column: 1 },
            }),
            configuration: None,
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_privacy: Some(CheckerSetting::True),
                relative_path: PathBuf::from("packs/bar"),
                private_constants: vec![String::from("::Bar")]
                    .into_iter()
                    .collect(),
                ..default_defining_pack()
            }),
            referencing_pack: default_referencing_pack(),
            expected_violation: None,
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_privacy_constants_includes_parent_of_referenced_constant(
    ) -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: Some(Reference {
                constant_name: String::from("::Bar::BarChild"),
                defining_pack_name: Some(String::from("packs/bar")),
                referencing_pack_name: String::from("packs/foo"),
                relative_referencing_file: String::from(
                    "packs/foo/app/services/foo.rb",
                ),
                relative_defining_file: Some(String::from(
                    "packs/bar/app/services/bar.rb",
                )),
                source_location: SourceLocation { line: 3, column: 1 },
            }),
            configuration: None,
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_privacy: Some(CheckerSetting::True),
                relative_path: PathBuf::from("packs/bar"),
                private_constants: vec![String::from("::Bar")]
                    .into_iter()
                    .collect(),
                ..default_defining_pack()
            }),
            referencing_pack: default_referencing_pack(),
            expected_violation: Some(build_expected_violation_with_constant(
                String::from("packs/foo/app/services/foo.rb:3:1\nPrivacy violation: `::Bar::BarChild` is private to `packs/bar`, but referenced from `packs/foo`"),
                String::from("privacy"), false,
                String::from("::Bar::BarChild")
            )),
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_privacy_constants_match_full_namespace() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: Some(Reference {
                constant_name: String::from("::Barbie::BarChild"),
                defining_pack_name: Some(String::from("packs/bar")),
                referencing_pack_name: String::from("packs/foo"),
                relative_referencing_file: String::from(
                    "packs/foo/app/services/foo.rb",
                ),
                relative_defining_file: Some(String::from(
                    "packs/bar/app/api/bar.rb",
                )),
                source_location: SourceLocation { line: 3, column: 1 },
            }),
            configuration: None,
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_privacy: Some(CheckerSetting::True),
                relative_path: PathBuf::from("packs/bar"),
                private_constants: vec![String::from("::Bar")]
                    .into_iter()
                    .collect(),
                public_folder: Some(PathBuf::from("packs/bar/app/public")),
                ..default_defining_pack()
            }),
            referencing_pack: default_referencing_pack(),
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_private_constants_does_not_include_referenced_constant(
    ) -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: Some(Reference {
                constant_name: String::from("::Bar"),
                defining_pack_name: Some(String::from("packs/bar")),
                referencing_pack_name: String::from("packs/foo"),
                relative_referencing_file: String::from(
                    "packs/foo/app/services/foo.rb",
                ),
                relative_defining_file: Some(String::from(
                    "packs/bar/app/api/bar.rb",
                )),
                source_location: SourceLocation { line: 3, column: 1 },
            }),
            configuration: None,
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_privacy: Some(CheckerSetting::True),
                relative_path: PathBuf::from("packs/bar"),
                private_constants: vec![String::from("::DifferentConstant")]
                    .into_iter()
                    .collect(),
                public_folder: Some(PathBuf::from("packs/bar/app/public")),
                ..default_defining_pack()
            }),
            referencing_pack: default_referencing_pack(),
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_private_constants_does_include_referenced_public_constant(
    ) -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: Some(Reference {
                constant_name: String::from("::Bar"),
                defining_pack_name: Some(String::from("packs/bar")),
                referencing_pack_name: String::from("packs/foo"),
                relative_referencing_file: String::from(
                    "packs/foo/app/services/foo.rb",
                ),
                relative_defining_file: Some(String::from(
                    "packs/bar/app/public/bar.rb",
                )),
                source_location: SourceLocation { line: 3, column: 1 },
            }),
            configuration: None,
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                enforce_privacy: Some(CheckerSetting::True),
                relative_path: PathBuf::from("packs/bar"),
                private_constants: vec![String::from("::Bar")]
                    .into_iter()
                    .collect(),
                public_folder: Some(PathBuf::from("packs/bar/app/public")),
                ..default_defining_pack()
            }),
            referencing_pack: default_referencing_pack(),
            ..Default::default()
        };
        test_check(&Checker {}, &mut test_checker)
    }

    #[test]
    fn test_defining_pack_not_found() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: Some(Reference {
                constant_name: String::from("::Bar"),
                defining_pack_name: Some(String::from("packs/bar")),
                referencing_pack_name: String::from("packs/foo"),
                relative_referencing_file: String::from(
                    "packs/foo/app/services/foo.rb",
                ),
                relative_defining_file: Some(String::from(
                    "packs/bar/app/public/bar.rb",
                )),
                source_location: SourceLocation { line: 3, column: 1 },
            }),
            configuration: None,
            defining_pack: None,
            referencing_pack: default_referencing_pack(),
            ..Default::default()
        };
        let result = test_check(&Checker {}, &mut test_checker);
        assert!(result.is_err());
        Ok(())
    }
}
