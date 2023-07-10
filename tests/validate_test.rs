use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};

mod common;

#[test]
fn test_validate_cycle_detection() -> Result<(), Box<dyn Error>> {
    let expected_message = String::from(
        "
Found 1 strongly connected components (i.e. dependency cycles)
The following groups of packages form a cycle:

packs/foo, packs/bar",
    );

    Command::cargo_bin("packs")
        .unwrap()
        .arg("--project-root")
        .arg("tests/fixtures/app_with_dependency_cycles")
        .arg("--debug")
        .arg("validate")
        .assert()
        .failure()
        .stdout(predicate::str::contains("1 validation error(s) detected:"))
        .stdout(predicate::str::contains(expected_message));

    common::teardown();
    Ok(())
}
