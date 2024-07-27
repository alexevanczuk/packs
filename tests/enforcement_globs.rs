use std::error::Error;

use assert_cmd::Command;

mod common;

#[test]
fn test_check() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("pks")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app_with_enforcement_globs")
        .arg("--debug")
        .arg("check")
        .assert()
        .success();
    common::teardown();
    Ok(())
}
