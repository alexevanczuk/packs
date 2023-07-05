use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};
mod common;

#[test]
fn test_update() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("list_monkey_patches")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "`String` was monkey patched in the following files:\n  - config/initializers/string_and_date_extensions.rb\n"
        ))
        .stdout(predicate::str::contains(
            "`Date` was monkey patched in the following files:\n  - config/initializers/string_and_date_extensions.rb\n"
        ));

    Ok(())
}
