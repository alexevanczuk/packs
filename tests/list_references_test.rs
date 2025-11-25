use assert_cmd::prelude::*;
use std::{error::Error, fs, process::Command};
use tempfile::TempDir;
mod common;

#[test]
fn test_list_references_simple_app() -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let output_file = temp_dir.path().join("references.json");

    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--experimental-parser")
        .arg("list-references")
        .arg("--out")
        .arg(&output_file)
        .assert()
        .success();

    let contents = fs::read_to_string(&output_file)?;
    let json: serde_json::Value = serde_json::from_str(&contents)?;

    let expected: serde_json::Value = serde_json::json!({
        "packs/foo/app/services/foo.rb": {
            "::Bar": "packs/bar/app/services/bar.rb"
        }
    });

    assert_eq!(json, expected);

    Ok(())
}

#[test]
fn test_list_references_namespaced_app() -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let output_file = temp_dir.path().join("references.json");

    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/app_with_namespaced_tests")
        .arg("--experimental-parser")
        .arg("list-references")
        .arg("--out")
        .arg(&output_file)
        .assert()
        .success();

    let contents = fs::read_to_string(&output_file)?;
    let json: serde_json::Value = serde_json::from_str(&contents)?;

    let expected: serde_json::Value = serde_json::json!({
        "spec/models/some_module/some_other_module/some_class_spec.rb": {
            "::SomeModule::SomeOtherModule::SomeClass": "app/models/some_module/some_other_module/some_class.rb"
        }
    });

    assert_eq!(json, expected);

    Ok(())
}
