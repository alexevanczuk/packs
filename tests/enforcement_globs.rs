use std::error::Error;

use assert_cmd::Command;
#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;

mod common;

#[test]
fn test_check() -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app_with_enforcement_globs")
        .arg("--debug")
        .arg("check")
        .assert()
        .success();
    common::teardown();
    Ok(())
}
