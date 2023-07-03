use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, path::Path, process::Command};
mod common;

#[test]
fn test_update() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Successfully updated package_todo.yml files!",
        ));

    let package_todo_yml_filepath =
        Path::new("tests/fixtures/simple_app/packs/foo/package_todo.yml");
    let actual = std::fs::read_to_string(package_todo_yml_filepath)?;
    let expected = String::from(
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
    - privacy
    files:
    - packs/foo/app/services/foo.rb
",
    );
    std::fs::remove_file(package_todo_yml_filepath)?;
    assert_eq!(expected, actual);

    common::teardown();

    Ok(())
}

#[test]
fn test_update_with_experimental_parser() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")
        .unwrap()
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("--experimental-parser")
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Successfully updated package_todo.yml files!",
        ));

    let package_todo_yml_filepath =
        Path::new("tests/fixtures/simple_app/packs/foo/package_todo.yml");
    let actual = std::fs::read_to_string(package_todo_yml_filepath)?;
    let expected = String::from(
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
    - privacy
    files:
    - packs/foo/app/services/foo.rb
",
    );
    std::fs::remove_file(package_todo_yml_filepath)?;
    assert_eq!(expected, actual);

    common::teardown();

    Ok(())
}
