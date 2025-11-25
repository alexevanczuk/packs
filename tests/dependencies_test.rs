use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};

mod common;

#[test]
fn test_list_pack_dependencies_with_explicit_dependencies(
) -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("list-pack-dependencies")
        .arg("packs/baz")
        .assert()
        .success()
        .stdout(predicate::str::contains("Explicit (1):"))
        .stdout(predicate::str::contains("packs/foo"));

    common::teardown();
    Ok(())
}

#[test]
fn list_pack_dependencies_with_implicit_dependencies(
) -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/contains_package_todo")
        .arg("--debug")
        .arg("list-pack-dependencies")
        .arg("packs/bar")
        .assert()
        .success()
        .stdout(predicate::str::contains("Explicit (0):"))
        .stdout(predicate::str::contains("packs/foo"))
        .stdout(predicate::str::contains("dependency: 1"));

    common::teardown();
    Ok(())
}
