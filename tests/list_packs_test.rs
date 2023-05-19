use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};

#[test]
fn lint_packs() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs-rs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_dependency_violation")
        .arg("list-packs")
        .assert()
        .success()
        .stdout(predicate::str::contains("package.yml"))
        .stdout(predicate::str::contains("packs/bar/package.yml"))
        .stdout(predicate::str::contains("packs/foo/package.yml"));
    Ok(())
}
