use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};

mod common;

#[test]
fn test_list_pack_dependents_with_public_dependents(
) -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("list-pack-dependents")
        .arg("packs/baz")
        .assert()
        .success()
        .stdout(predicate::str::contains("Public dependents (1):"))
        .stdout(predicate::str::contains("packs/foo"));

    common::teardown();
    Ok(())
}

#[test]
fn test_list_pack_dependents_with_violation_dependents(
) -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/contains_package_todo")
        .arg("--debug")
        .arg("list-pack-dependents")
        .arg("packs/bar")
        .assert()
        .success()
        .stdout(predicate::str::contains("Public dependents (0):"))
        .stdout(predicate::str::contains("packs/foo"))
        .stdout(predicate::str::contains("dependency: 1"));

    common::teardown();
    Ok(())
}
