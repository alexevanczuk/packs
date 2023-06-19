use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use tracing::debug;

use super::{Configuration, Violation};

#[derive(PartialEq, Debug, Eq, Deserialize, Serialize, Default, Clone)]
pub struct ViolationGroup {
    // Use serde rename to parse the key as violations
    #[serde(rename = "violations")]
    pub violation_types: HashSet<String>,
    #[serde(serialize_with = "serialize_sorted_set")]
    pub files: HashSet<String>,
}

fn serialize_sorted_set<S>(
    set: &HashSet<String>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut sorted_files: Vec<&String> = set.iter().collect();
    sorted_files.sort();
    sorted_files.serialize(serializer)
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize, Default, Clone)]
pub struct PackageTodo {
    #[serde(flatten, serialize_with = "serialize_violations_by_defining_pack")]
    pub violations_by_defining_pack:
        HashMap<String, HashMap<String, ViolationGroup>>,
}

fn serialize_violations_by_defining_pack<S>(
    map: &HashMap<String, HashMap<String, ViolationGroup>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut sorted_entries: Vec<_> = map.iter().collect();
    sorted_entries.sort_by_key(|(k, _)| k.as_str());

    let mut map_serializer =
        serializer.serialize_map(Some(sorted_entries.len()))?;

    for (key, value) in sorted_entries {
        let quoted_value: HashMap<_, _> = value
            .iter()
            .map(|(k, v)| (format!("QUOTE{}QUOTE", k), v))
            .collect();
        map_serializer.serialize_entry(key, &quoted_value)?;
    }

    map_serializer.end()
}

pub fn package_todos_for_pack_name(
    violations_by_responsible_pack_name: HashMap<String, Vec<Violation>>,
) -> HashMap<String, PackageTodo> {
    let mut ret = HashMap::new();

    // Then we group violations by the defining pack, since that's how they're grouped in the package_todo.yml file
    for (responsible_pack_name, mut violations) in
        violations_by_responsible_pack_name
    {
        let mut violations_by_defining_pack: HashMap<
            String,
            HashMap<String, ViolationGroup>,
        > = HashMap::new();
        // Sort violations by the defining pack name, then constant name, then file name
        // This ensures they show up deterministically in the package_todo.yml file.
        violations.sort_by(|a, b| {
            let defining_pack_name_comparison = a
                .identifier
                .defining_pack_name
                .cmp(&b.identifier.defining_pack_name);
            if defining_pack_name_comparison != std::cmp::Ordering::Equal {
                return defining_pack_name_comparison;
            }

            let constant_name_comparison =
                a.identifier.constant_name.cmp(&b.identifier.constant_name);
            if constant_name_comparison != std::cmp::Ordering::Equal {
                return constant_name_comparison;
            }

            a.identifier.file.cmp(&b.identifier.file)
        });

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
                .insert(violation.identifier.file.to_owned());
            violation_group
                .violation_types
                .insert(violation.identifier.violation_type.to_owned());
        }

        let package_todo = PackageTodo {
            violations_by_defining_pack,
        };

        ret.insert(responsible_pack_name, package_todo);
    }

    ret
}
pub fn write_violations_to_disk(
    configuration: Configuration,
    violations: Vec<Violation>,
) {
    debug!("Starting writing violations to disk");
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

    let package_todos_by_pack_name =
        package_todos_for_pack_name(violations_by_responsible_pack);

    for (responsible_pack_name, package_todo) in package_todos_by_pack_name {
        write_package_todo_to_disk(
            &configuration,
            responsible_pack_name,
            package_todo,
        );
    }

    debug!("Finished writing violations to disk");
}

fn serialize_package_todo(
    responsible_pack_name: &String,
    package_todo: PackageTodo,
) -> String {
    let package_todo_yml = serde_yaml::to_string(&package_todo).unwrap();

    // This is a hack until I figure out how to use serde to do this for me
    let package_todo_yml = package_todo_yml.replace("QUOTE", "\"");
    let header = header(&responsible_pack_name);
    let package_todo_yml = header + &package_todo_yml;
    package_todo_yml
}

fn write_package_todo_to_disk(
    configuration: &Configuration,
    responsible_pack_name: String,
    package_todo: PackageTodo,
) {
    let responsible_pack =
        configuration.pack_set.for_pack(&responsible_pack_name);
    let package_todo_yml_absolute_filepath = responsible_pack
        .yml
        .parent()
        .unwrap()
        .join("package_todo.yml");

    if !package_todo_yml_absolute_filepath.exists() {
        std::fs::File::create(&package_todo_yml_absolute_filepath).unwrap();
    }

    let package_todo_yml =
        serialize_package_todo(&responsible_pack_name, package_todo);

    std::fs::write(package_todo_yml_absolute_filepath, package_todo_yml)
        .unwrap();
}

fn header(responsible_pack_name: &String) -> String {
    format!("\
# This file contains a list of dependencies that are not part of the long term plan for the
# '{}' package.
# We should generally work to reduce this list over time.
#
# You can regenerate this file using the following command:
#
# bin/packwerk update-todo
---
", responsible_pack_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_deserialize_trivial_case() {
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
        let mut violation_types = HashSet::new();
        violation_types.insert(String::from("dependency"));
        let mut files = HashSet::new();
        files.insert(String::from("packs/foo/app/services/foo.rb"));
        bar_violations.insert(
            String::from("::Bar"),
            ViolationGroup {
                violation_types,
                files,
            },
        );

        let mut violation_types = HashSet::new();
        violation_types.insert(String::from("dependency"));
        violation_types.insert(String::from("privacy"));
        let mut files = HashSet::new();
        files.insert(String::from("packs/foo/app/services/foo.rb"));
        bar_violations.insert(
            String::from("::Baz"),
            ViolationGroup {
                violation_types,
                files,
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
