use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use std::{error::Error, fs, path::PathBuf, process::Command};
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

    // Write some dummy file to `tmp/cache/packwerk` to simulate a cache.
    let cache_dir =
        PathBuf::from("tests/fixtures/simple_app/tmp/cache/packwerk");

    fs::create_dir_all(&cache_dir)?;
    let dummy_file = cache_dir.join("dummy_file");
    fs::write(dummy_file, "dummy file")?;

    assert!(!is_tmp_cache_packwerk_empty().unwrap());

    Command::new(cargo_bin!("packs"))
        .arg("--debug")
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("delete-cache")
        .assert()
        .success();
    common::teardown();

    assert!(is_tmp_cache_packwerk_empty().unwrap());

    Ok(())
}
