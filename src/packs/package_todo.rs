use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap, HashSet};
use tracing::debug;

use super::{pack::Pack, Configuration, Violation};

#[derive(PartialEq, Debug, Eq, Deserialize, Serialize, Default, Clone)]
pub struct ViolationGroup {
    // Use serde rename to parse the key as violations
    #[serde(rename = "violations", serialize_with = "serialize_sorted_set")]
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
        BTreeMap<String, BTreeMap<String, ViolationGroup>>,
}

fn serialize_violations_by_defining_pack<S>(
    map: &BTreeMap<String, BTreeMap<String, ViolationGroup>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map_serializer = serializer.serialize_map(Some(map.len()))?;

    for (key, value) in map {
        let mut quoted_sorted_violations_by_constant: BTreeMap<
            String,
            ViolationGroup,
        > = BTreeMap::new();
        for (constant_name, violation_group) in value {
            // HACK: This is the first part of a hack (search `HACK:` for more)
            let quoted_constant_name = format!("#{}#", constant_name);

            // The issue is that I have not been able to figure out how to get serde to serialize
            // a String key with double quotes.
            // When I tried this:
            // let quoted_constant_name = format!("\"{}\"", constant_name);
            // serde_yaml would escape the quotes, so I would get this:
            // '\"::Bar\"'
            // (uncomment the above and run tests to reproduce)
            quoted_sorted_violations_by_constant
                .insert(quoted_constant_name, violation_group.clone());
        }
        let modified_key = if key == &String::from(".") {
            String::from("#.#")
        } else {
            key.to_owned()
        };

        map_serializer.serialize_entry(
            &modified_key,
            &quoted_sorted_violations_by_constant,
        )?;
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
        let mut violations_by_defining_pack: BTreeMap<
            String,
            BTreeMap<String, ViolationGroup>,
        > = BTreeMap::new();
        // Sort violations by the defining pack name, then constant name, then file name
        // This ensures they show up deterministically in the package_todo.yml file.
        violations.sort_by(|a, b| {
            a.identifier
                .defining_pack_name
                .cmp(&b.identifier.defining_pack_name)
                .then_with(|| {
                    a.identifier.constant_name.cmp(&b.identifier.constant_name)
                })
                .then_with(|| a.identifier.file.cmp(&b.identifier.file))
        });

        for violation in violations {
            let defining_pack_name =
                violation.identifier.defining_pack_name.to_owned();
            let existing_violations_by_constant_group =
                violations_by_defining_pack
                    .entry(defining_pack_name)
                    .or_default();

            let violation_group = existing_violations_by_constant_group
                .entry(violation.identifier.constant_name.to_owned())
                .or_default();

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
    configuration: &Configuration,
    violations: HashSet<Violation>,
) {
    debug!("Starting writing violations to disk");
    // First we need to group the violations by the repsonsible pack, which today is always the referencing pack
    // Later if we change where a violation shows up, we should delegate to the checker
    // to decide what pack it should be in.
    let mut violations_by_responsible_pack: HashMap<String, Vec<Violation>> =
        HashMap::new();
    for violation in violations {
        if violation.identifier.strict {
            continue;
        }
        let referencing_pack_name =
            violation.identifier.referencing_pack_name.to_owned();
        violations_by_responsible_pack
            .entry(referencing_pack_name)
            .or_default()
            .push(violation);
    }

    let package_todos_by_pack_name =
        package_todos_for_pack_name(violations_by_responsible_pack);

    let all_packs = &configuration.pack_set.packs;
    all_packs.par_iter().for_each(|p| {
        let package_todo = package_todos_by_pack_name.get(&p.name);
        match package_todo {
            Some(package_todo) => write_package_todo_to_disk(
                p,
                package_todo,
                configuration.packs_first_mode,
            ),
            None => delete_package_todo_from_disk(p),
        }
    });

    debug!("Finished writing violations to disk");
}

fn serialize_package_todo(
    responsible_pack_name: &String,
    package_todo: &PackageTodo,
    packs_first_mode: bool,
) -> String {
    let package_todo_yml = serde_yaml::to_string(&package_todo).unwrap();

    // HACK: This is the other part of the hack above (search `HACK:` for more)
    let package_todo_yml = package_todo_yml.replace("'#", "\"");
    let package_todo_yml = package_todo_yml.replace("#'", "\"");
    let header = header(responsible_pack_name, packs_first_mode);
    header + &package_todo_yml
}

fn write_package_todo_to_disk(
    responsible_pack: &Pack,
    package_todo: &PackageTodo,
    packs_first_mode: bool,
) {
    let package_todo_yml_absolute_filepath = responsible_pack
        .yml
        .parent()
        .unwrap()
        .join("package_todo.yml");

    if !package_todo_yml_absolute_filepath.exists() {
        std::fs::File::create(&package_todo_yml_absolute_filepath).unwrap();
    }

    let package_todo_yml = serialize_package_todo(
        &responsible_pack.name,
        package_todo,
        packs_first_mode,
    );

    std::fs::write(package_todo_yml_absolute_filepath, package_todo_yml)
        .unwrap();
}

fn delete_package_todo_from_disk(responsible_pack: &Pack) {
    let package_todo_yml_absolute_filepath = responsible_pack
        .yml
        .parent()
        .unwrap()
        .join("package_todo.yml");

    if package_todo_yml_absolute_filepath.exists() {
        // Delete package_todo_yml_absolute_filepath
        std::fs::remove_file(package_todo_yml_absolute_filepath).unwrap();
    }
}

fn header(responsible_pack_name: &String, packs_first_mode: bool) -> String {
    let command = if packs_first_mode {
        "pks update"
    } else {
        "bin/packwerk update-todo"
    };

    format!("\
# This file contains a list of dependencies that are not part of the long term plan for the
# '{}' package.
# We should generally work to reduce this list over time.
#
# You can regenerate this file using the following command:
#
# {}
---
", responsible_pack_name, command)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn construct_violations(
        constant_name: String,
        input_types: Vec<String>,
        input_files: Vec<String>,
    ) -> BTreeMap<String, ViolationGroup> {
        let mut bar_violations = BTreeMap::new();
        let mut files = HashSet::new();
        let mut violation_types = HashSet::new();

        for file in input_files {
            files.insert(file);
        }

        for violation_type in input_types {
            violation_types.insert(violation_type);
        }

        bar_violations.insert(
            constant_name,
            ViolationGroup {
                violation_types,
                files,
            },
        );

        bar_violations
    }

    fn bar_violations() -> BTreeMap<String, ViolationGroup> {
        construct_violations(
            String::from("::Bar"),
            vec![String::from("dependency")],
            vec![String::from("packs/foo/app/services/foo.rb")],
        )
    }

    fn bar_blah_violations() -> BTreeMap<String, ViolationGroup> {
        construct_violations(
            String::from("::BarBlah"),
            vec![String::from("dependency")],
            vec![String::from("packs/foo/app/services/foo.rb")],
        )
    }

    fn baz_violations() -> BTreeMap<String, ViolationGroup> {
        construct_violations(
            String::from("::Baz"),
            vec![String::from("dependency"), String::from("privacy")],
            vec![String::from("packs/foo/app/services/foo.rb")],
        )
    }

    fn example_package_todo(defining_package_name: String) -> PackageTodo {
        let mut violations_by_defining_pack: BTreeMap<
            String,
            BTreeMap<String, ViolationGroup>,
        > = BTreeMap::new();
        let bar_violations = bar_violations();
        let bar_blah_violations = bar_blah_violations();
        let baz_violations = baz_violations();
        let mut merged_map: BTreeMap<String, ViolationGroup> = BTreeMap::new();
        merged_map.extend(bar_violations);
        merged_map.extend(bar_blah_violations);
        merged_map.extend(baz_violations);

        violations_by_defining_pack.insert(defining_package_name, merged_map);

        PackageTodo {
            violations_by_defining_pack,
        }
    }

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
            \"::BarBlah\":
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

        let expected = example_package_todo(String::from("packs/bar"));

        let actual: PackageTodo = serde_yaml::from_str(&contents).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_serialize_trivial_case() {
        let expected: String = String::from(
            "\
# This file contains a list of dependencies that are not part of the long term plan for the
# 'packs/foo' package.
# We should generally work to reduce this list over time.
#
# You can regenerate this file using the following command:
#
# bin/packwerk update-todo
---
packs/bar:
  \"::Bar\":
    violations:
    - dependency
    files:
    - packs/foo/app/services/foo.rb
  \"::BarBlah\":
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

        let actual_package_todo =
            example_package_todo(String::from("packs/bar"));
        let actual = serialize_package_todo(
            &String::from("packs/foo"),
            &actual_package_todo,
            false,
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_serialize_violation_against_root() {
        let expected: String = String::from(
            "\
# This file contains a list of dependencies that are not part of the long term plan for the
# 'packs/foo' package.
# We should generally work to reduce this list over time.
#
# You can regenerate this file using the following command:
#
# bin/packwerk update-todo
---
\".\":
  \"::Bar\":
    violations:
    - dependency
    files:
    - packs/foo/app/services/foo.rb
  \"::BarBlah\":
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

        let actual_package_todo = example_package_todo(String::from("."));
        let actual = serialize_package_todo(
            &String::from("packs/foo"),
            &actual_package_todo,
            false,
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_serialize_trivial_case_in_packs_first_mode() {
        let expected: String = String::from(
            "\
# This file contains a list of dependencies that are not part of the long term plan for the
# 'packs/foo' package.
# We should generally work to reduce this list over time.
#
# You can regenerate this file using the following command:
#
# pks update
---
packs/bar:
  \"::Bar\":
    violations:
    - dependency
    files:
    - packs/foo/app/services/foo.rb
  \"::BarBlah\":
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

        let actual_package_todo =
            example_package_todo(String::from("packs/bar"));
        let actual = serialize_package_todo(
            &String::from("packs/foo"),
            &actual_package_todo,
            true,
        );

        assert_eq!(expected, actual);
    }
}
