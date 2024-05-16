use std::error::Error;

use assert_cmd::Command;
use predicates::prelude::*;

mod common;

#[test]
fn test_check() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app_with_enforcement_globs")
        .arg("--debug")
        .arg("check")
        .assert()
        .failure();
        //.stdout(predicate::str::contains("2 violation(s) detected:"))
        //.stdout(predicate::str::contains("packs/foo/app/services/foo.rb:3:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."))
        //.stdout(predicate::str::contains("packs/foo/app/services/foo.rb:3:4\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"));

    common::teardown();
    Ok(())
}
