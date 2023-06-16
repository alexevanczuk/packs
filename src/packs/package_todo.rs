use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Configuration, Violation};

#[derive(PartialEq, Debug, Eq, Deserialize, Serialize, Default, Clone)]
pub struct ViolationGroup {
    // Use serde rename to parse the key as violations
    #[serde(rename = "violations")]
    pub violation_types: Vec<String>,
    pub files: Vec<String>,
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize, Default, Clone)]
pub struct PackageTodo {
    #[serde(flatten)]
    pub violations_by_defining_pack:
        HashMap<String, HashMap<String, ViolationGroup>>,
}

pub fn write_violations_to_disk(
    configuration: Configuration,
    violations: Vec<Violation>,
) {
    // First we need to group the violations by the repsonsible pack, which today is always the referencing pack
    // Later if we change where a violation shows up, we should delegate to the checker
    // to decide what pack it should be in.
    let mut violations_by_responsible_pack: HashMap<String, Vec<Violation>> =
        HashMap::new();
    for violation in violations {
        let referencing_pack_name =
            violation.identifier.referencing_pack_name.to_owned();
        violations_by_responsible_pack
            .entry(referencing_pack_name)
            .or_insert(Vec::new())
            .push(violation);
    }

    // Then we group violations by the defining pack, since that's how they're grouped in the package_todo.yml file
    for (responsible_pack_name, violations) in violations_by_responsible_pack {
        let mut violations_by_defining_pack: HashMap<
            String,
            HashMap<String, ViolationGroup>,
        > = HashMap::new();
        for violation in violations {
            let defining_pack_name =
                violation.identifier.defining_pack_name.to_owned();
            let existing_violations_by_constant_group =
                violations_by_defining_pack
                    .entry(defining_pack_name)
                    .or_insert(HashMap::new());

            let violation_group = existing_violations_by_constant_group
                .entry(violation.identifier.constant_name.to_owned())
                .or_insert(ViolationGroup::default());

            violation_group
                .files
                .push(violation.identifier.file.to_owned());
            violation_group
                .violation_types
                .push(violation.identifier.violation_type.to_owned());
        }

        let package_todo = PackageTodo {
            violations_by_defining_pack,
        };

        let responsible_pack =
            configuration.pack_set.for_pack(&responsible_pack_name);
        let package_todo_yml_absolute_filepath = responsible_pack
            .yml
            .parent()
            .unwrap()
            .join("package_todo.yml");

        let package_todo_yml = serde_yaml::to_string(&package_todo).unwrap();

        if !package_todo_yml_absolute_filepath.exists() {
            std::fs::File::create(&package_todo_yml_absolute_filepath).unwrap();
        }

        std::fs::write(package_todo_yml_absolute_filepath, package_todo_yml)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_trivial_case() {
        let contents: String = String::from(
            "
        # This file contains a list of dependencies that are not part of the long term plan for the
        # 'packs/foo' package.
        # We should generally work to reduce this list over time.
        #
        # You can regenerate this file using the following command:
        #
        # bin/packwerk update-todo
        packs/bar:
            \"::Bar\":
                violations:
                - dependency
                files:
                - packs/foo/app/services/foo.rb
            \"::Baz\":
                violations:
                - dependency
                - privacy
                files:
                - packs/foo/app/services/foo.rb
        ",
        );

        let mut violations_by_defining_pack: HashMap<
            String,
            HashMap<String, ViolationGroup>,
        > = HashMap::new();
        let mut bar_violations = HashMap::new();
        bar_violations.insert(
            String::from("::Bar"),
            ViolationGroup {
                violation_types: vec![String::from("dependency")],
                files: vec![String::from("packs/foo/app/services/foo.rb")],
            },
        );
        bar_violations.insert(
            String::from("::Baz"),
            ViolationGroup {
                violation_types: vec![
                    String::from("dependency"),
                    String::from("privacy"),
                ],
                files: vec![String::from("packs/foo/app/services/foo.rb")],
            },
        );
        violations_by_defining_pack
            .insert(String::from("packs/bar"), bar_violations);
        let expected = PackageTodo {
            violations_by_defining_pack,
        };

        let actual: PackageTodo = serde_yaml::from_str(&contents).unwrap();
        assert_eq!(expected, actual);
    }
}
