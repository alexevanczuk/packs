use super::{CheckerInterface, ViolationIdentifier};
use crate::packs::checker::Reference;
use crate::packs::Violation;

pub struct Checker {}

// TODO:
// Once we implement packs validate, we need to ensure that nothing can add a dependency
// from a pack to a pack that doesn't permit visibility from the referencing pack
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

        if defining_pack.enforce_visibility.is_false() {
            return None;
        }

        if defining_pack.visible_to.contains(referencing_pack_name) {
            return None;
        }

        let defining_pack_name = &defining_pack.name;

        if relative_defining_file.is_none() {
            return None;
        }

        if referencing_pack_name == defining_pack_name {
            return None;
        }

        let message = format!(
            "{}:{}:{}\nVisibility violation: `{}` belongs to `{}`, which is not visible to `{}`",
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
    use std::collections::HashSet;

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
            enforce_visibility: CheckerSetting::True,
            ..Pack::default()
        };
        let referencing_pack = Pack {
            name: String::from("packs/foo"),
            ..Pack::default()
        };

        let reference = Reference {
            constant_name: String::from("::Foo"),
            defining_pack: Some(&defining_pack),
            referencing_pack: &referencing_pack,
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
    fn reference_is_a_visibility_violation() {
        let checker = Checker {};

        let defining_pack = Pack {
            name: String::from("packs/foo"),
            enforce_visibility: CheckerSetting::True,
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

        let expected_violation = Violation {
            message: String::from("packs/bar/app/services/bar.rb:3:1\nVisibility violation: `::Foo` belongs to `packs/foo`, which is not visible to `packs/bar`"),
            identifier: ViolationIdentifier {
                violation_type: String::from("visibility"),
                file: String::from("packs/bar/app/services/bar.rb"),
                constant_name: String::from("::Foo"),
                referencing_pack_name: String::from("packs/bar"),
                defining_pack_name: String::from("packs/foo"),
            },
        };
        assert_eq!(expected_violation, checker.check(&reference).unwrap())
    }

    #[test]
    fn reference_is_not_a_visibility_violation() {
        let checker = Checker {};

        let mut visible_to = HashSet::new();
        visible_to.insert(String::from("packs/bar"));

        let defining_pack = Pack {
            name: String::from("packs/foo"),
            visible_to,
            enforce_visibility: CheckerSetting::True,
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
