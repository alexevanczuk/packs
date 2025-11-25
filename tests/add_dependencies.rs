use assert_cmd::prelude::*;
use packs::packs::pack::Pack; // I'm definitely doing the wrong thing here
use std::{error::Error, fs, process::Command};
mod common;

// TODO
// We should validate here and blow up with a helpful error message if adding dependencies causes a circular dependency
#[test]
fn test_check_add_dependencies() -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/app_with_missing_dependencies")
        .arg("--debug")
        .arg("add-dependencies")
        .arg("packs/baz")
        .assert()
        .success();

    let after_pack: Pack = serde_yaml::from_str(
    &fs::read_to_string("tests/fixtures/app_with_missing_dependencies/packs/baz/package.yml")
        .expect("Failed to read package.yml"),
)
.expect("Failed to deserialize package.yml");

    let expected_dependencies: std::collections::HashSet<String> =
        vec!["packs/bar".to_string()].into_iter().collect();

    assert_eq!(after_pack.dependencies, expected_dependencies);

    common::teardown();
    common::set_up_fixtures();

    Ok(())
}
