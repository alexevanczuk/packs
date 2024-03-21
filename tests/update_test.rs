use assert_cmd::prelude::*;
use predicates::prelude::*;
use serial_test::serial;
use std::{error::Error, path::Path, process::Command};
mod common;
use pretty_assertions::assert_eq;

#[test]
// This and the next test are run in serial because they both use the same fixtures.
#[serial]
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
#[serial]
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

#[test]
fn test_update_with_stale_violations() -> Result<(), Box<dyn Error>> {
    common::set_up_fixtures();

    Command::cargo_bin("packs")
        .unwrap()
        .arg("--project-root")
        .arg("tests/fixtures/contains_stale_violations")
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Successfully updated package_todo.yml files!",
        ));

    let package_todo_yml_filepath = Path::new(
        "tests/fixtures/contains_stale_violations/packs/foo/package_todo.yml",
    );
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
    - privacy
    files:
    - packs/foo/app/services/foo.rb
",
    );

    assert_eq!(expected, actual);

    let package_todo_yml_filepath = Path::new(
        "tests/fixtures/contains_stale_violations/packs/bar/package_todo.yml",
    );
    assert!(!package_todo_yml_filepath.exists());
    common::set_up_fixtures();

    Ok(())
}

#[test]
fn test_update_with_packs_first_app() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_packs_first_app")
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Successfully updated package_todo.yml files!",
        ));

    let package_todo_yml_filepath = Path::new(
        "tests/fixtures/simple_packs_first_app/packs/foo/package_todo.yml",
    );
    let actual = std::fs::read_to_string(package_todo_yml_filepath)?;
    let expected = String::from(
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
fn test_update_with_strict_violations() -> anyhow::Result<()> {
    let path = Path::new(
        "tests/fixtures/contains_strict_violations/packs/foo/package_todo.yml",
    );
    let _ignore = std::fs::remove_file(path);

    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/contains_strict_violations")
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Successfully updated package_todo.yml files!",
        ));

    assert!(
        !path.exists(),
        "todo should not be created for strict violations"
    );
    Ok(())
}
