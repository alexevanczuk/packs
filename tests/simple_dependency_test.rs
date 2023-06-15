use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, fs::File, io::Write, process::Command};

#[test]
fn test_check() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("1 violation(s) detected:"));
    // Commented out until we start reading package_todo.yml files
    // .stdout(predicate::str::contains("dependency: packs/foo/app/services/foo.rb:3 references ::Bar from packs/bar without an explicit dependency in packs/foo/package.yml"));
    Ok(())
}

#[test]
#[ignore]
fn test_check_with_package_todo_file() -> Result<(), Box<dyn Error>> {
    let contents: String = String::from(
    "\
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
    - tests/fixtures/simple_app/packs/foo/app/services/foo.rb
    ",
    );

    let mut file =
        File::create("tests/fixtures/simple_app/packs/foo/package_todo.yml")?;
    file.write_all(contents.as_bytes())?;

    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("No violations detected!"));

    std::fs::remove_file(
        "tests/fixtures/simple_app/packs/foo/package_todo.yml",
    )?;

    Ok(())
}
