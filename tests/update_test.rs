use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};

#[test]
#[ignore]
fn test_update() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Successfully updated package_todo.yml files!",
        ));

    // TODO: assert that the package_todo.yml files were updated
    // Then clean them up
    Ok(())
}
