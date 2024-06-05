use assert_cmd::prelude::*;
use std::{error::Error, process::Command};

mod common;
#[test]
fn test_check() -> Result<(), Box<dyn Error>> {
    let output = Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/visibility_violations")
        .arg("--debug")
        .arg("check")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let stripped_output =
        String::from_utf8_lossy(&strip_ansi_escapes::strip(output)).to_string();

    assert!(stripped_output.contains("1 violation(s) detected:"));
    dbg!(&stripped_output);
    assert!(stripped_output.contains("detected:\npacks/baz/app/services/baz.rb:3:4\nVisibility violation: `::Foo` belongs to `packs/foos/foo`, which is not visible to `packs/baz`"));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_disabled_enforce_visibility() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/visibility_violations")
        .arg("--debug")
        .arg("--disable-enforce-visibility")
        .arg("check")
        .assert()
        .success();

    common::teardown();
    Ok(())
}
