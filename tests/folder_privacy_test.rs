use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};
mod common;

#[test]
fn test_check() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("pks")?
        .arg("--project-root")
        .arg("tests/fixtures/folder_privacy_violations")
        .arg("--debug")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Folder Privacy violation: `::Foo` belongs to `packs/foos/foo`, which is private to `packs/baz` as it is not a sibling pack or parent pack."));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_with_overrides() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("pks")?
        .arg("--project-root")
        .arg("tests/fixtures/folder_privacy_violations_with_overrides")
        .arg("--debug")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Folder Privacy violation: `::Foo` belongs to the `packs/foos/foo` product service, which is not visible to `packs/baz` as it is a different product service. See https://docs.google.com"));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_enforce_folder_privacy_disabled() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("pks")?
        .arg("--project-root")
        .arg("tests/fixtures/folder_privacy_violations")
        .arg("--debug")
        .arg("--disable-enforce-folder-privacy")
        .arg("check")
        .assert()
        .success();

    common::teardown();
    Ok(())
}

#[test]
fn test_invisible_pack_violation_with_deprecated_enforce_folder_visibility(
) -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("pks")?
        .arg("--project-root")
        .arg("tests/fixtures/folder_visibility_violations")
        .arg("--debug")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Folder Privacy violation: `::Foo` belongs to `packs/foos/foo`, which is private to `packs/baz` as it is not a sibling pack or parent pack."));

    common::teardown();
    Ok(())
}
