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
            defining_pack_name: String::from("packs/Foo"),
            referencing_pack_name: String::from("packs/foo"),
            source_location: SourceLocation { line: 3, column: 1 },
        };
        assert_eq!(None, checker.check(&configuration, &reference))
    }

    #[test]
    #[ignore]
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
            defining_pack_name: String::from("packs/bar"),
            referencing_pack_name: String::from("packs/foo"),
            source_location: SourceLocation { line: 3, column: 1 },
        };
        let expected_violation = Violation {
            message: String::from("dependency: packs/foo/app/services/foo.rb:3 references Bar from packs/bar without an explicit dependency in packs/foo/package.yml"),
        };
        assert_eq!(
            expected_violation,
            checker.check(&configuration, &reference).unwrap()
        )
    }
}
