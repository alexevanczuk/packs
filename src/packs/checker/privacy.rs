use super::{CheckerInterface, ViolationIdentifier};
use crate::packs::checker::Reference;
use crate::packs::Violation;

pub struct Checker {}

impl CheckerInterface for Checker {
    fn check(&self, reference: &Reference) -> Option<Violation> {
        let referencing_pack = reference.referencing_pack.clone();

        let referencing_pack_name = &referencing_pack.name;
        let defining_pack_name = reference.defining_pack_name.clone()?;

        if referencing_pack_name == &defining_pack_name {
            return None;
        }

        let ignored_dependency = referencing_pack
            .ignored_dependencies
            .contains(&defining_pack_name);
        if !referencing_pack_dependencies.contains(&defining_pack_name)
            && !ignored_dependency
        {
            let message = format!(
                // "dependency: packs/foo/app/services/foo.rb:3 references Bar from packs/bar without an explicit dependency in packs/foo/package.yml"
                "dependency: {}:{} references {} from {} without an explicit dependency in {}/package.yml",
                reference.relative_referencing_file,
                reference.source_location.line,
                reference.constant_name,
                defining_pack_name,
                referencing_pack_name,
            );
            let violation_type = String::from("dependency");
            let file = reference.relative_referencing_file.clone();
            let identifier = ViolationIdentifier {
                violation_type,
                file,
                constant_name: reference.constant_name.clone(),
                referencing_pack_name: referencing_pack_name.clone(),
                defining_pack_name,
            };

            return Some(Violation {
                message,
                identifier,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs::*;
    use std::{path::PathBuf, sync::Arc};

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
            referencing_pack: Arc::new(
                configuration.pack_set.for_pack(&String::from("packs/foo")),
            ),
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
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
            referencing_pack: Arc::new(
                configuration.pack_set.for_pack(&String::from("packs/foo")),
            ),
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        let expected_violation = Violation {
            message: String::from("dependency: packs/foo/app/services/foo.rb:3 references ::Bar from packs/bar without an explicit dependency in packs/foo/package.yml"),
            identifier: ViolationIdentifier {
                violation_type: String::from("dependency"),
                file: String::from("packs/foo/app/services/foo.rb"),
                constant_name: String::from("::Bar"),
                referencing_pack_name: String::from("packs/foo"),
                defining_pack_name: String::from("packs/bar"),
            },
        };
        assert_eq!(expected_violation, checker.check(&&reference).unwrap())
    }
}
