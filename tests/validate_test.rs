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
        .stdout(predicate::str::contains("2 validation error(s) detected:"))
        .stdout(predicate::str::contains(expected_message))
        .stdout(predicate::str::contains(
            "Package cannot list itself as a dependency: packs/baz/package.yml",
        ));

    common::teardown();
    Ok(())
}

#[test]
fn test_validate_layer() -> Result<(), Box<dyn Error>> {
    let expected_message_1 = String::from(
        "\'layer\' must be specified in \'packs/baz/package.yml\' because `enforce_layers` is true or strict.",
    );
    let expected_message_2 = String::from(
        "Invalid \'layer\' option in \'packs/foo/package.yml\'. `layer` must be one of the layers defined in `packwerk.yml`"
    );

    Command::cargo_bin("packs")
        .unwrap()
        .arg("--project-root")
        .arg("tests/fixtures/app_with_layer_violations_in_yml")
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
