use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};

#[test]
fn test_generate_cache() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("pks")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("generate-cache")
        .arg("packs/bar/app/services/bar.rb")
        .arg("packs/foo/app/services/foo.rb")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Cache was generated for files packs/bar/app/services/bar.rb and packs/foo/app/services/foo.rb\n",
        ));
    Ok(())
}
