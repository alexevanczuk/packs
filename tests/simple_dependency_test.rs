use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};

#[test]
fn test_validate() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_dependency_violation")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("1 violation detected:"))
        .stdout(predicate::str::contains("dependency: packs/foo/app/services/foo.rb:3 references Bar from packs/bar without an explicit dependency in packs/foo/package.yml"));
    Ok(())
}
