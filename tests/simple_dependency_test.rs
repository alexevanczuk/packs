use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};

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
