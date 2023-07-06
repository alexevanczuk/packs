use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};

mod common;

#[test]
fn test_check() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("2 violation(s) detected:"))
        .stdout(predicate::str::contains("packs/foo/app/services/foo.rb:3:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."))
        .stdout(predicate::str::contains("packs/foo/app/services/foo.rb:3:4\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_with_package_todo_file() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/contains_package_todo")
        .arg("--debug")
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("No violations detected!"));

    common::teardown();

    Ok(())
}

#[test]
fn test_check_with_experimental_parser() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")
        .unwrap()
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--experimental-parser")
        .arg("--debug")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("2 violation(s) detected:"))
        .stdout(predicate::str::contains("packs/foo/app/services/foo.rb:3:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."))
        .stdout(predicate::str::contains("packs/foo/app/services/foo.rb:3:4\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_cycle_detection() -> Result<(), Box<dyn Error>> {
    let expected_message = String::from(
        "
Found 1 strongly connected components (i.e. dependency cycles)
The following groups of packages from a cycle:

packs/foo, packs/bar",
    );

    Command::cargo_bin("packs")
        .unwrap()
        .arg("--project-root")
        .arg("tests/fixtures/app_with_dependency_cycles")
        .arg("--debug")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("1 validation error(s) detected:"))
        .stdout(predicate::str::contains(expected_message));

    common::teardown();
    Ok(())
}
