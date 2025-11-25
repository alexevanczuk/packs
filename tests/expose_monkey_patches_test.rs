#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};
mod common;

#[test]
fn test_expose_monkey_patches() -> Result<(), Box<dyn Error>> {
    // For the full test, see src/packs/monkey_patch_detection.rs
    // This just ensures the CLI is hooked up correctly to the internal API
    let expected_message_portion = String::from(
        "The following is a list of constants that are redefined by your app.",
    );
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/app_with_monkey_patches")
        .arg("--experimental-parser")
        .arg("expose-monkey-patches")
        .arg("--rubydir=tests/fixtures/app_with_monkey_patches/rubydir_stub")
        .arg("--gemdir=tests/fixtures/app_with_monkey_patches/gemdir_stub")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_message_portion));

    Ok(())
}
