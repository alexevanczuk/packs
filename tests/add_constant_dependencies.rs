use assert_cmd::Command;
#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use serial_test::serial;
use std::{collections::HashSet, path::PathBuf};

mod common;

#[test]
#[serial]
fn test_add_constant_dependencies() -> anyhow::Result<()> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/app_with_missing_dependencies")
        .arg("update-dependencies-for-constant")
        .arg("::Bar::Tender")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Successfully updated 1 dependency for constant '::Bar::Tender'",
        ));

    let config = packs::packs::configuration(
        PathBuf::from("tests/fixtures/app_with_missing_dependencies"),
        &0,
    )
    .unwrap();

    let pack = config.pack_set.for_pack("packs/foo").unwrap();
    assert_eq!(pack.dependencies.len(), 0);

    let pack = config.pack_set.for_pack("packs/baz").unwrap();
    assert_eq!(pack.dependencies.len(), 1);

    let mut expected = HashSet::new();
    expected.insert("packs/bar".to_owned());
    assert_eq!(pack.dependencies, expected);
    common::teardown();
    common::set_up_fixtures();

    Ok(())
}

#[test]
#[serial]
fn test_add_constant_dependencies_no_dependencies() -> anyhow::Result<()> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/app_with_missing_dependencies")
        .arg("update-dependencies-for-constant")
        .arg("::Bar::Nope")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No dependencies to update for constant '::Bar::Nope'",
        ));

    Ok(())
}
