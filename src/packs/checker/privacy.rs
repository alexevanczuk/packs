use super::{CheckerInterface, ViolationIdentifier};
use crate::packs::checker::Reference;
use crate::packs::Violation;

pub struct Checker {}

impl CheckerInterface for Checker {
    fn check(&self, reference: &Reference) -> Option<Violation> {
        let referencing_pack = &reference.referencing_pack;
        let relative_defining_file = &reference.relative_defining_file;

        let referencing_pack_name = &referencing_pack.name;
        let defining_pack = &reference.defining_pack;
        if defining_pack.is_none() {
            return None;
        }
        let defining_pack = defining_pack.unwrap();

        if defining_pack.enforce_privacy.is_false() {
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
        let is_public = relative_defining_file
            .as_ref()
            .unwrap()
            .contains("/public/");

        if is_public {
            return None;
        }

        let message = format!(
            // "dependency: packs/foo/app/services/foo.rb:3 references Bar from packs/bar without an explicit dependency in packs/foo/package.yml"
            "privacy: {}:{} references private constant {} from {}",
            reference.relative_referencing_file,
            reference.source_location.line,
            reference.constant_name,
            defining_pack_name,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs::*;

    #[test]
    fn referencing_and_defining_pack_are_identical() {
        let checker = Checker {};

        let defining_pack = Pack {
            name: String::from("packs/foo"),
            enforce_privacy: CheckerSetting::True,
            ..Pack::default()
        };

        let referencing_pack = &Pack {
            name: String::from("packs/foo"),
            ..Pack::default()
        };
        let reference = Reference {
            constant_name: String::from("::Foo"),
            defining_pack: Some(&defining_pack),
            referencing_pack,
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/bar/app/services/bar.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };
        assert_eq!(None, checker.check(&reference))
    }

    #[test]
    fn test_check() {
        let checker = Checker {};
        let defining_pack = Pack {
            name: String::from("packs/bar"),
            enforce_privacy: CheckerSetting::True,
            ..Pack::default()
        };

        let referencing_pack = &Pack {
            name: String::from("packs/foo"),
            ..Pack::default()
        };

        let reference = Reference {
            constant_name: String::from("::Bar"),
            defining_pack: Some(&defining_pack),
            referencing_pack,
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/bar/app/services/bar.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        let expected_violation = Violation {
            message: String::from("privacy: packs/foo/app/services/foo.rb:3 references private constant ::Bar from packs/bar"),
            identifier: ViolationIdentifier {
                violation_type: String::from("privacy"),
                file: String::from("packs/foo/app/services/foo.rb"),
                constant_name: String::from("::Bar"),
                referencing_pack_name: String::from("packs/foo"),
                defining_pack_name: String::from("packs/bar"),
            },
        };
        assert_eq!(expected_violation, checker.check(&reference).unwrap())
    }

    #[test]
    fn test_ignored_private_constants() {
        let checker = Checker {};
        let defining_pack = Pack {
            name: String::from("packs/foo"),
            enforce_privacy: CheckerSetting::True,
            ignored_private_constants: HashSet::from([String::from("::Foo")]),
            ..Pack::default()
        };

        let referencing_pack = Pack {
            name: String::from("packs/bar"),
            ..Pack::default()
        };
        let reference = Reference {
            constant_name: String::from("::Foo"),
            defining_pack: Some(&defining_pack),
            referencing_pack: &referencing_pack,
            relative_referencing_file: String::from(
                "packs/bar/app/services/bar.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/foo/app/services/foo.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        assert_eq!(None, checker.check(&reference))
    }
}
