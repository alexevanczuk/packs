use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};

#[test]
fn test_generate_cache() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("pks")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("generate-cache")
        .arg("file1.rb")
        .arg("file2.rb")
        .arg("file3.rb")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Cache was generated for files file1.rb, file2.rb, and file3.rb",
        ));
    Ok(())
}
