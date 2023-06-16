use assert_cmd::prelude::*;
use std::{error::Error, process::Command};

#[test]
#[ignore]
fn test_generate_cache() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("generate_cache")
        .arg("packs/bar/app/services/bar.rb")
        .arg("packs/foo/app/services/foo.rb")
        .assert()
        .success();
    Ok(())
}
