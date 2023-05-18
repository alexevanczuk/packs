use assert_cmd::prelude::*;
use std::{error::Error, path::Path, process::Command};

#[test]
fn test_validate() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/valid_project")
        .arg("validate")
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_generate() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/valid_project")
        .arg("--codeowners-file-path")
        .arg("../../../tmp/CODEOWNERS")
        .arg("generate")
        .assert()
        .success();

    let expected_codeowners: String = std::fs::read_to_string(Path::new("tests/fixtures/valid_project/.github/CODEOWNERS"))?;
    let actual_codeowners: String = std::fs::read_to_string(Path::new("tmp/CODEOWNERS"))?;

    assert_eq!(expected_codeowners, actual_codeowners);

    Ok(())
}
