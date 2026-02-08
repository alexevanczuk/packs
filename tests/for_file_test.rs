#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};

#[test]
fn for_file_in_pack() -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("for-file")
        .arg("packs/foo/app/services/foo.rb")
        .assert()
        .success()
        .stdout(predicate::str::contains("packs/foo/package.yml"));
    Ok(())
}

#[test]
fn for_file_in_root() -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("for-file")
        .arg("app/services/some_root_class.rb")
        .assert()
        .success()
        .stdout(predicate::str::contains("package.yml"))
        .stdout(predicate::str::contains("packs/").not());
    Ok(())
}

#[test]
fn for_file_not_found() -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("for-file")
        .arg("nonexistent/file.rb")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No pack found"));
    Ok(())
}
