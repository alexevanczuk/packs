#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use predicates::prelude::*;
mod common;

#[test]
fn test_check_with_corrupt_todo() -> anyhow::Result<()> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/contains_corrupt_todo")
        .arg("--debug")
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to deserialize the package_todo.yml"))
        .stderr(predicate::str::contains("Try deleting the file and running the `update` command to regenerate it"));

    common::teardown();
    Ok(())
}
