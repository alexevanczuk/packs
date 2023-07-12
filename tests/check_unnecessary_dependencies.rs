use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};
mod common;

#[test]
fn test_check_unnecessary_dependencies() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/app_with_dependency_cycles")
        .arg("--debug")
        .arg("check-unnecessary-dependencies")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "packs/bar depends on packs/foo but does not use it",
        ))
        .stdout(predicate::str::contains(
            "packs/foo depends on packs/bar but does not use it",
        ));
    Ok(())
}

#[test]
fn test_check_unnecessary_dependencies_no_issue() -> Result<(), Box<dyn Error>>
{
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("check-unnecessary-dependencies")
        .assert()
        .success();
    Ok(())
}
