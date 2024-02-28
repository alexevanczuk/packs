use assert_cmd::Command;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use serial_test::serial;
use std::{collections::HashSet, error::Error, path::PathBuf};

mod common;

#[test]
#[serial]
fn test_add_dependency() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/app_with_missing_dependency")
        .arg("add-dependency")
        .arg("packs/baz")
        .arg("packs/foo")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Successfully added `packs/foo` as a dependency to `packs/baz`!",
        ));

    let config = packs::packs::configuration(PathBuf::from(
        "tests/fixtures/app_with_missing_dependency",
    ))
    .unwrap();

    let pack = config.pack_set.for_pack("packs/baz").unwrap();

    assert_eq!(pack.dependencies.len(), 1);
    let mut expected = HashSet::new();
    expected.insert("packs/foo".to_owned());
    assert_eq!(pack.dependencies, expected);
    common::teardown();
    common::set_up_fixtures();

    Ok(())
}

#[test]
#[serial]
fn test_add_dependency_creating_cycle() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/app_with_missing_dependency")
        .arg("add-dependency")
        .arg("packs/bar")
        .arg("packs/foo")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Added `packs/foo` as a dependency to `packs/bar`!",
        ))
        .stdout(predicate::str::contains("Warning: This creates a cycle!"))
        .stdout(predicate::str::contains(
            "Found 1 strongly connected components (i.e. dependency cycles)",
        ))
        .stdout(predicate::str::contains(
            "The following groups of packages form a cycle:",
        ))
        .stdout(predicate::str::contains("packs/foo, packs/bar"));

    let config = packs::packs::configuration(PathBuf::from(
        "tests/fixtures/app_with_missing_dependency",
    ))
    .unwrap();

    let pack = config.pack_set.for_pack("packs/bar").unwrap();

    assert_eq!(pack.dependencies.len(), 1);
    let mut expected = HashSet::new();
    expected.insert("packs/foo".to_owned());
    assert_eq!(pack.dependencies, expected);
    common::teardown();
    common::set_up_fixtures();

    Ok(())
}

#[test]
fn test_add_dependency_unnecessarily() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/app_with_missing_dependency")
        .arg("add-dependency")
        .arg("packs/foo")
        .arg("packs/bar")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "`packs/foo` already depends on `packs/bar`!",
        ));

    let config = packs::packs::configuration(PathBuf::from(
        "tests/fixtures/app_with_missing_dependency",
    ))
    .unwrap();

    let pack = config.pack_set.for_pack("packs/foo").unwrap();

    assert_eq!(pack.dependencies.len(), 1);
    let mut expected = HashSet::new();
    expected.insert("packs/bar".to_owned());
    assert_eq!(pack.dependencies, expected);
    common::teardown();
    Ok(())
}
