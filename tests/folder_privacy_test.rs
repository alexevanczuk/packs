use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};
mod common;

#[test]
fn test_invisible_pack_violation() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/folder_privacy_violations")
        .arg("--debug")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Folder Visibility violation: `::Foo` belongs to `packs/foos/foo`, which is not visible to `packs/baz` as it is not a sibling pack or parent pack."));

    common::teardown();
    Ok(())
}

#[test]
fn test_invisible_pack_violation_with_deprecated_enforce_folder_visibility(
) -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/folder_visibility_violations")
        .arg("--debug")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Folder Visibility violation: `::Foo` belongs to `packs/foos/foo`, which is not visible to `packs/baz` as it is not a sibling pack or parent pack."));

    common::teardown();
    Ok(())
}
