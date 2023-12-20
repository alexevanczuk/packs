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

#[test]
fn test_validate_architecture() -> Result<(), Box<dyn Error>> {
    let expected_message_1 = String::from(
        "
Invalid 'dependencies' in 'packs/baz/package.yml'. 'packs/baz/package.yml' has a layer type of 'technical_services,' which cannot rely on 'packs/bar,' which has a layer type of 'admin.' `architecture_layers` can be found in packwerk.yml",
    );
    let expected_message_2 = String::from(
        "
Invalid 'dependencies' in 'packs/foo/package.yml'. 'packs/foo/package.yml' has a layer type of 'product,' which cannot rely on 'packs/bar,' which has a layer type of 'admin.' `architecture_layers` can be found in packwerk.yml",
    );

    Command::cargo_bin("packs")
        .unwrap()
        .arg("--project-root")
        .arg("tests/fixtures/app_with_architecture_violations_in_yml")
        .arg("validate")
        .assert()
        .failure()
        .stdout(predicate::str::contains("2 validation error(s) detected:"))
        .stdout(predicate::str::contains(expected_message_1))
        .stdout(predicate::str::contains(expected_message_2));

    common::teardown();
    Ok(())
}

#[test]
fn test_validate_with_referencing_unknown_pack() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/references_unknown_pack")
        .arg("--debug")
        .arg("validate")
        .assert()
        .failure()
        .stdout(predicate::str::contains("has \'packs/unknown-pack\' in its dependencies, but that pack cannot be found"));

    common::teardown();
    Ok(())
}
