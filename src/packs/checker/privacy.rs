use super::{get_defining_pack, CheckerInterface, ViolationIdentifier};
use crate::packs::checker::Reference;
use crate::packs::{Configuration, Violation};

pub struct Checker {}

impl CheckerInterface for Checker {
    fn check(
        &self,
        reference: &Reference,
        configuration: &Configuration,
    ) -> Option<Violation> {
        let referencing_pack =
            &reference.referencing_pack(&configuration.pack_set);
        let relative_defining_file = &reference.relative_defining_file;

        let referencing_pack_name = &referencing_pack.name;
        let defining_pack = &reference.defining_pack(&configuration.pack_set);
        if defining_pack.is_none() {
            return None;
        }
        let defining_pack = defining_pack.unwrap();

        if defining_pack.enforce_privacy().is_false() {
            return None;
        }

        if defining_pack
            .ignored_private_constants
            .contains(&reference.constant_name)
        {
            return None;
        }

        let defining_pack_name = &defining_pack.name;

        if relative_defining_file.is_none() {
            return None;
        }

        if referencing_pack_name == defining_pack_name {
            return None;
        }

        // This is a hack for now – we need to read package.yml file public_paths at some point,
        // and probably find a better way to check if the constant is public

        let public_folder = &defining_pack.public_folder();

        let is_public = relative_defining_file
            .as_ref()
            .unwrap()
            .starts_with(public_folder.to_string_lossy().as_ref());

        // Note this means that if the constant is ALSO in the list of private_constants,
        // it will be considered public.
        // This is how packwerk does it today.
        // Later we might want to add some sort of validation that a constant can be in the public folder OR in the list of private_constants,
        // but not both.
        if is_public {
            return None;
        }

        let private_constants = &defining_pack.private_constants;

        if !private_constants.is_empty() {
            let constant_is_private =
                private_constants.contains(&reference.constant_name);

            let constant_is_in_private_namespace =
                private_constants.iter().any(|private_constant| {
                    reference.constant_name.starts_with(private_constant)
                });

            if !constant_is_private && !constant_is_in_private_namespace {
                return None;
            }
        }

        // START: Original packwerk message
        // path/to/file.rb:36:0
        // Privacy violation: '::Constant' is private to 'packs/defining_pack' but referenced from 'packs/referencing_pack'.
        // Is there a public entrypoint in 'packs/defining_pack/app/public/' that you can use instead?

        // Inference details: this is a reference to ::Constant which seems to be defined in packs/defining_pack/path/to/definition.rb.
        // To receive help interpreting or resolving this error message, see: https://github.com/Shopify/packwerk/blob/main/TROUBLESHOOT.md#Troubleshooting-violations
        // END: Original packwerk message

        let message = format!(
            "{}:{}:{}\nPrivacy violation: `{}` is private to `{}`, but referenced from `{}`",
            reference.relative_referencing_file,
            reference.source_location.line,
            reference.source_location.column,
            reference.constant_name,
            defining_pack_name,
            referencing_pack_name,
        );

        let violation_type = String::from("privacy");
        let file = reference.relative_referencing_file.clone();
        let identifier = ViolationIdentifier {
            violation_type,
            file,
            constant_name: reference.constant_name.clone(),
            referencing_pack_name: referencing_pack_name.clone(),
            defining_pack_name: defining_pack_name.clone(),
        };

        Some(Violation {
            message,
            identifier,
        })
    }

    fn is_strict_mode_violation(
        &self,
        violation: &ViolationIdentifier,
        configuration: &Configuration,
    ) -> bool {
        let defining_pack =
            get_defining_pack(violation, &configuration.pack_set);

        defining_pack.enforce_privacy().is_strict()
    }

    fn violation_type(&self) -> String {
        "privacy".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;
    use crate::packs::{
        pack::{CheckerSetting, Pack},
        *,
    };

    #[test]
    fn referencing_and_defining_pack_are_identical() {
        let checker = Checker {};

        let defining_pack = Pack {
            name: String::from("packs/foo"),
            enforce_privacy: Some(CheckerSetting::True),
            ..Pack::default()
        };

        let referencing_pack = Pack {
            name: String::from("packs/foo"),
            ..Pack::default()
        };
        let reference = Reference {
            constant_name: String::from("::Foo"),
            defining_pack_name: Some(defining_pack.name.to_owned()),
            referencing_pack_name: referencing_pack.name.to_owned(),
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/bar/app/services/bar.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        let root_pack = Pack {
            name: String::from("."),
            ..Pack::default()
        };

        let configuration = Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![
                    root_pack,
                    defining_pack,
                    referencing_pack,
                ]),
                HashMap::new(),
            ),
            ..Configuration::default()
        };

        assert_eq!(None, checker.check(&reference, &configuration))
    }

    #[test]
    fn test_check() {
        let checker = Checker {};
        let defining_pack = Pack {
            name: String::from("packs/bar"),
            enforce_privacy: Some(CheckerSetting::True),
            public_folder: Some(PathBuf::from("packs/bar/app/public")),
            ..Pack::default()
        };

        let referencing_pack = Pack {
            name: String::from("packs/foo"),
            ..Pack::default()
        };

        let reference = Reference {
            constant_name: String::from("::Bar"),
            defining_pack_name: Some(defining_pack.name.to_owned()),
            referencing_pack_name: referencing_pack.name.to_owned(),
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/bar/app/services/bar.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        let expected_violation = Violation {
            message: String::from("packs/foo/app/services/foo.rb:3:1\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"),
            identifier: ViolationIdentifier {
                violation_type: String::from("privacy"),
                file: String::from("packs/foo/app/services/foo.rb"),
                constant_name: String::from("::Bar"),
                referencing_pack_name: String::from("packs/foo"),
                defining_pack_name: String::from("packs/bar"),
            },
        };

        let root_pack = Pack {
            name: String::from("."),
            ..Pack::default()
        };

        let configuration = Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![
                    root_pack,
                    defining_pack,
                    referencing_pack,
                ]),
                HashMap::new(),
            ),
            ..Configuration::default()
        };

        assert_eq!(
            expected_violation,
            checker.check(&reference, &configuration).unwrap()
        )
    }

    #[test]
    fn test_ignored_private_constants() {
        let checker = Checker {};
        let defining_pack = Pack {
            name: String::from("packs/foo"),
            enforce_privacy: Some(CheckerSetting::True),
            ignored_private_constants: HashSet::from([String::from("::Foo")]),
            ..Pack::default()
        };

        let referencing_pack = Pack {
            name: String::from("packs/bar"),
            ..Pack::default()
        };

        let root_pack = Pack {
            name: String::from("."),
            ..Pack::default()
        };

        let reference = Reference {
            constant_name: String::from("::Foo"),
            defining_pack_name: Some(defining_pack.name.to_owned()),
            referencing_pack_name: referencing_pack.name.to_owned(),
            relative_referencing_file: String::from(
                "packs/bar/app/services/bar.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/foo/app/services/foo.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        let configuration = Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![
                    root_pack,
                    defining_pack,
                    referencing_pack,
                ]),
                HashMap::new(),
            ),
            ..Configuration::default()
        };

        assert_eq!(None, checker.check(&reference, &configuration))
    }

    #[test]
    fn test_public_folder_detection_works() {
        let checker = Checker {};
        let defining_pack = Pack {
            name: String::from("packs/bar"),
            enforce_privacy: Some(CheckerSetting::True),
            public_folder: Some(PathBuf::from("packs/bar/app/public")),
            ..Pack::default()
        };

        let referencing_pack = Pack {
            name: String::from("packs/foo"),
            ..Pack::default()
        };

        let reference = Reference {
            constant_name: String::from("::Bar"),
            defining_pack_name: Some(defining_pack.name.to_owned()),
            referencing_pack_name: referencing_pack.name.to_owned(),
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/bar/app/services/public/bar.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        let expected_violation = Violation {
            message: String::from("packs/foo/app/services/foo.rb:3:1\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"),
            identifier: ViolationIdentifier {
                violation_type: String::from("privacy"),
                file: String::from("packs/foo/app/services/foo.rb"),
                constant_name: String::from("::Bar"),
                referencing_pack_name: String::from("packs/foo"),
                defining_pack_name: String::from("packs/bar"),
            },
        };

        let root_pack = Pack {
            name: String::from("."),
            ..Pack::default()
        };

        let configuration = Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![
                    root_pack,
                    defining_pack,
                    referencing_pack,
                ]),
                HashMap::new(),
            ),
            ..Configuration::default()
        };

        assert_eq!(
            expected_violation,
            checker.check(&reference, &configuration).unwrap()
        )
    }

    #[test]
    fn test_custom_public_folder_detection_works() {
        let checker = Checker {};
        let defining_pack = Pack {
            name: String::from("packs/bar"),
            public_folder: Some(PathBuf::from("packs/bar/app/api")),
            enforce_privacy: Some(CheckerSetting::True),
            ..Pack::default()
        };

        let referencing_pack = Pack {
            name: String::from("packs/foo"),
            ..Pack::default()
        };

        let reference = Reference {
            constant_name: String::from("::Bar"),
            defining_pack_name: Some(defining_pack.name.to_owned()),
            referencing_pack_name: referencing_pack.name.to_owned(),
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/bar/app/api/bar.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        let root_pack = Pack {
            name: String::from("."),
            ..Pack::default()
        };

        let configuration = Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![
                    root_pack,
                    defining_pack,
                    referencing_pack,
                ]),
                HashMap::new(),
            ),
            ..Configuration::default()
        };

        assert_eq!(None, checker.check(&reference, &configuration))
    }

    #[test]
    fn test_private_constants_includes_referenced_constant() {
        let checker = Checker {};
        let defining_pack = Pack {
            name: String::from("packs/bar"),
            private_constants: vec![String::from("::Bar")]
                .into_iter()
                .collect(),
            enforce_privacy: Some(CheckerSetting::True),
            public_folder: Some(PathBuf::from("packs/bar/app/public")),
            ..Pack::default()
        };

        let referencing_pack = Pack {
            name: String::from("packs/foo"),
            ..Pack::default()
        };

        let reference = Reference {
            constant_name: String::from("::Bar"),
            defining_pack_name: Some(defining_pack.name.to_owned()),
            referencing_pack_name: referencing_pack.name.to_owned(),
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/bar/app/api/bar.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        let expected_violation = Violation {
            message: String::from("packs/foo/app/services/foo.rb:3:1\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"),
            identifier: ViolationIdentifier {
                violation_type: String::from("privacy"),
                file: String::from("packs/foo/app/services/foo.rb"),
                constant_name: String::from("::Bar"),
                referencing_pack_name: String::from("packs/foo"),
                defining_pack_name: String::from("packs/bar"),
            },
        };

        let root_pack = Pack {
            name: String::from("."),
            ..Pack::default()
        };

        let configuration = Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![
                    root_pack,
                    defining_pack,
                    referencing_pack,
                ]),
                HashMap::new(),
            ),
            ..Configuration::default()
        };

        assert_eq!(
            expected_violation,
            checker.check(&reference, &configuration).unwrap()
        )
    }

    #[test]
    fn test_private_constants_includes_parent_of_referenced_constant() {
        let checker = Checker {};
        let defining_pack = Pack {
            name: String::from("packs/bar"),
            private_constants: vec![String::from("::Bar")]
                .into_iter()
                .collect(),
            enforce_privacy: Some(CheckerSetting::True),
            public_folder: Some(PathBuf::from("packs/bar/app/public")),
            ..Pack::default()
        };

        let referencing_pack = Pack {
            name: String::from("packs/foo"),
            ..Pack::default()
        };

        let reference = Reference {
            constant_name: String::from("::Bar::BarChild"),
            defining_pack_name: Some(defining_pack.name.to_owned()),
            referencing_pack_name: referencing_pack.name.to_owned(),
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/bar/app/api/bar.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        let expected_violation = Violation {
            message: String::from("packs/foo/app/services/foo.rb:3:1\nPrivacy violation: `::Bar::BarChild` is private to `packs/bar`, but referenced from `packs/foo`"),
            identifier: ViolationIdentifier {
                violation_type: String::from("privacy"),
                file: String::from("packs/foo/app/services/foo.rb"),
                constant_name: String::from("::Bar::BarChild"),
                referencing_pack_name: String::from("packs/foo"),
                defining_pack_name: String::from("packs/bar"),
            },
        };

        let root_pack = Pack {
            name: String::from("."),
            ..Pack::default()
        };

        let configuration = Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![
                    root_pack,
                    defining_pack,
                    referencing_pack,
                ]),
                HashMap::new(),
            ),
            ..Configuration::default()
        };

        assert_eq!(
            expected_violation,
            checker.check(&reference, &configuration).unwrap()
        )
    }

    #[test]
    fn test_private_constants_does_not_include_referenced_constant() {
        let checker = Checker {};
        let defining_pack = Pack {
            name: String::from("packs/bar"),
            private_constants: vec![String::from("::DifferentConstant")]
                .into_iter()
                .collect(),
            enforce_privacy: Some(CheckerSetting::True),
            ..Pack::default()
        };

        let referencing_pack = Pack {
            name: String::from("packs/foo"),
            ..Pack::default()
        };

        let reference = Reference {
            constant_name: String::from("::Bar"),
            defining_pack_name: Some(defining_pack.name.to_owned()),
            referencing_pack_name: referencing_pack.name.to_owned(),
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/bar/app/api/bar.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        let root_pack = Pack {
            name: String::from("."),
            ..Pack::default()
        };

        let configuration = Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![
                    root_pack,
                    defining_pack,
                    referencing_pack,
                ]),
                HashMap::new(),
            ),
            ..Configuration::default()
        };
        assert_eq!(None, checker.check(&reference, &configuration))
    }

    #[test]
    fn test_private_constants_does_include_referenced_public_constant() {
        let checker = Checker {};
        let defining_pack = Pack {
            name: String::from("packs/bar"),
            private_constants: vec![String::from("::Bar")]
                .into_iter()
                .collect(),
            enforce_privacy: Some(CheckerSetting::True),
            public_folder: Some(PathBuf::from("packs/bar/app/public")),
            ..Pack::default()
        };

        let referencing_pack = Pack {
            name: String::from("packs/foo"),
            ..Pack::default()
        };

        let root_pack = Pack {
            name: String::from("."),
            ..Pack::default()
        };

        let reference = Reference {
            constant_name: String::from("::Bar"),
            defining_pack_name: Some(defining_pack.name.to_owned()),
            referencing_pack_name: referencing_pack.name.to_owned(),
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/bar/app/public/bar.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        let configuration = Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![
                    root_pack,
                    defining_pack,
                    referencing_pack,
                ]),
                HashMap::new(),
            ),
            ..Configuration::default()
        };
        assert_eq!(None, checker.check(&reference, &configuration))
    }
}
