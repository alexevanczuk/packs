use super::{CheckerInterface, ViolationIdentifier};
use crate::packs::checker::Reference;
use crate::packs::Violation;

pub struct Checker {}

impl CheckerInterface for Checker {
    fn check(&self, reference: &Reference) -> Option<Violation> {
        let referencing_pack = &reference.referencing_pack;
        let relative_defining_file = &reference.relative_defining_file;
        dbg!(&relative_defining_file);

        if referencing_pack.enforce_privacy.is_false() {
            return None;
        }

        let referencing_pack_name = &referencing_pack.name;
        let defining_pack_name = &reference.defining_pack_name;

        if defining_pack_name.is_none() {
            return None;
        }

        if relative_defining_file.is_none() {
            return None;
        }

        let defining_pack_name = defining_pack_name.as_ref().unwrap();

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
    use std::path::PathBuf;

    #[test]
    fn referencing_and_defining_pack_are_identical() {
        let checker = Checker {};
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/simple_app")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
        );
        let reference = Reference {
            constant_name: String::from("::Foo"),
            defining_pack_name: Some(String::from("packs/foo")),
            referencing_pack: configuration
                .pack_set
                .for_pack(&String::from("packs/foo")),
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
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/simple_app")
                .canonicalize()
                .expect("Could not canonicalize path")
                .as_path(),
        );
        let reference = Reference {
            constant_name: String::from("::Bar"),
            defining_pack_name: Some(String::from("packs/bar")),
            referencing_pack: configuration
                .pack_set
                .for_pack(&String::from("packs/foo")),
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
}
