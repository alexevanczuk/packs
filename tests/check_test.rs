use assert_cmd::prelude::*;
use predicates::prelude::*;
use serial_test::serial;
use std::{error::Error, process::Command};

mod common;

#[test]
// The reason we need serial and below is because there is a race condition in the test where sometimes the cache directory
// does not exist, even though it is created before it is used.
// The reason for this is another test thread calls `teardown()` (or `delete_cache`) after these tests have created the cache
// directory. Eventually, we should fix this by improving test isolation so that we can continue to run these in parallel.
// Other tests that have the same issue can get the #[serial] tag until the issue is fixed.
#[serial]
fn test_check() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("2 violation(s) detected:"))
        .stdout(predicate::str::contains("packs/foo/app/services/foo.rb:3:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."))
        .stdout(predicate::str::contains("packs/foo/app/services/foo.rb:3:4\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"));

    common::teardown();
    Ok(())
}

#[test]
#[serial]
fn test_check_with_package_todo_file() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/contains_package_todo")
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("No violations detected!"));

    common::teardown();

    Ok(())
}

#[test]
#[serial]
#[should_panic(expected = "The experimental parser is coming soon!")]
fn test_check_with_experimental_parser() {
    Command::cargo_bin("packs")
        .unwrap()
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--experimental-parser")
        .arg("check")
        .assert()
        .success();

    common::teardown();
}
