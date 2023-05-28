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
        ))
        .stdout(predicate::str::contains(
            "The file content digests are {\"packs/bar/app/services/bar.rb\": \"f2af2fc657b71331ff3a8c39b48365eb\", \"packs/foo/app/services/foo.rb\": \"4be8effd7ac57323adcb53d0cf0ce789\"}",
        ));
    Ok(())
}
