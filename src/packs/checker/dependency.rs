use crate::packs::checker::Reference;
use crate::packs::{Configuration, Violation};

pub struct Checker {}

#[allow(unused_variables)]
impl Checker {
    pub fn check(
        &self,
        configuration: &Configuration,
        reference: &Reference,
    ) -> Option<Violation> {
        let referencing_pack = configuration
            .indexed_packs
            .get(&reference.referencing_pack_name)
            .unwrap();

        let defining_pack_name = reference.defining_pack_name.clone()?;

        if referencing_pack.name == defining_pack_name {
            return None;
        }

        let referencing_pack_dependencies = &configuration
            .indexed_packs
            .get(&reference.referencing_pack_name)
            .unwrap()
            .dependencies;

        if !referencing_pack_dependencies.contains(&defining_pack_name) {
            let message = format!(
                // "dependency: packs/foo/app/services/foo.rb:3 references Bar from packs/bar without an explicit dependency in packs/foo/package.yml"
                "dependency: {}:{} references {} from {} without an explicit dependency in {}/package.yml",
                reference.relative_referencing_file,
                reference.source_location.line,
                reference.constant_name,
                defining_pack_name,
                reference.referencing_pack_name,
            );
            return Some(Violation { message });
        }

        None
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
            referencing_pack_name: String::from("packs/foo"),
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            source_location: SourceLocation { line: 3, column: 1 },
        };
        assert_eq!(None, checker.check(&configuration, &reference))
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
            referencing_pack_name: String::from("packs/foo"),
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            source_location: SourceLocation { line: 3, column: 1 },
        };
        let expected_violation = Violation {
            message: String::from("dependency: packs/foo/app/services/foo.rb:3 references ::Bar from packs/bar without an explicit dependency in packs/foo/package.yml"),
        };
        assert_eq!(
            expected_violation,
            checker.check(&configuration, &reference).unwrap()
        )
    }
}
