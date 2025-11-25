use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, fs, process::Command};
use tempfile::TempDir;
mod common;

#[test]
fn test_list_references_json_output() -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let output_file = temp_dir.path().join("references.json");

    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--experimental-parser")
        .arg("list-references")
        .arg("--format")
        .arg("json")
        .arg("--out")
        .arg(&output_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Reference map written to"));

    // Read the output file
    let contents = fs::read_to_string(&output_file)?;

    // Parse as JSON to ensure it's valid
    let json: serde_json::Value = serde_json::from_str(&contents)?;

    // Verify it's an object (map)
    assert!(json.is_object());

    Ok(())
}

#[test]
fn test_list_references_text_output() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--experimental-parser")
        .arg("list-references")
        .arg("--format")
        .arg("text")
        .assert()
        .success()
        .stdout(predicate::str::contains("=>"));

    Ok(())
}

#[test]
fn test_list_references_default_json_format() -> Result<(), Box<dyn Error>> {
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

    // Read and parse the output
    let contents = fs::read_to_string(&output_file)?;
    let json: serde_json::Value = serde_json::from_str(&contents)?;

    // Verify it's valid JSON object
    assert!(json.is_object());

    // Verify default format is JSON (parseable)
    assert!(contents.starts_with("{"));

    Ok(())
}

#[test]
fn test_list_references_tracks_constant_references() -> Result<(), Box<dyn Error>> {
    let temp_dir = TempDir::new()?;
    let output_file = temp_dir.path().join("references.json");

    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--experimental-parser")
        .arg("list-references")
        .arg("--format")
        .arg("json")
        .arg("--out")
        .arg(&output_file)
        .assert()
        .success();

    let contents = fs::read_to_string(&output_file)?;
    let json: serde_json::Value = serde_json::from_str(&contents)?;

    // Verify the structure: should be a map where values are also maps
    if let Some(obj) = json.as_object() {
        for (_file, constants) in obj {
            // Each file should map to an object of constant->definition mappings
            assert!(constants.is_object(), "Constants should be an object");
        }
    }

    Ok(())
}

#[test]
fn test_list_references_invalid_format() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--experimental-parser")
        .arg("list-references")
        .arg("--format")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported format"));

    Ok(())
}
