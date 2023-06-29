use assert_cmd::prelude::*;
use std::{error::Error, fs, process::Command};
mod common;

fn is_tmp_cache_packwerk_empty() -> Result<bool, std::io::Error> {
    let directory = "tests/fixtures/simple_app/tmp/cache/packwerk";
    let dir_entries = fs::read_dir(directory);
    match dir_entries {
        Ok(mut dir_entries) => {
            let is_empty = dir_entries.next().is_none();
            Ok(is_empty)
        }
        Err(err) => match err.kind() {
            // The directory is empty if we can't find it.
            std::io::ErrorKind::NotFound => Ok(true),
            _ => Err(err),
        },
    }
}
#[test]
fn test_delete_cache() -> Result<(), Box<dyn Error>> {
    // Assert that `tmp/cache/packwerk` does *not* exist.
    // Is this test breaking for a seemingly unrelated change?
    // Make sure all tests are calling `teardown()` from `tests/common/mod.rs`
    assert!(is_tmp_cache_packwerk_empty().unwrap());

    // First, use the public API to generate the cache. That way we can verify our `delete_cache`
    // command under test is doing real work.
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("check")
        .arg("packs/bar/app/services/bar.rb")
        .arg("packs/foo/app/services/foo.rb")
        .assert()
        .failure();

    assert!(!is_tmp_cache_packwerk_empty().unwrap());

    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("delete_cache")
        .assert()
        .success();
    common::teardown();

    assert!(is_tmp_cache_packwerk_empty().unwrap());

    Ok(())
}
