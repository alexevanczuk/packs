use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};

mod common;

#[test]
fn test_check() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/architecture_violations")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("1 violation(s) detected:"))
        .stdout(predicate::str::contains("packs/feature_flags/app/services/feature_flags.rb:2:0\nArchitecture violation: `::Payments` belongs to `packs/payments` (whose layer is `product`) cannot be accessed from `packs/feature_flags` (whose layer is `utilities`)"));

    common::teardown();
    Ok(())
}
