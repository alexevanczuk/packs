use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command, fs};

mod common;

#[test]
fn init_pack() -> Result<(), Box<dyn Error>> {
    common::create_new_app();

    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/new_app")
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created "))
        .stdout(predicate::str::contains("tests/fixtures/new_app/packwerk.yml'"))
        .stdout(predicate::str::contains("tests/fixtures/new_app/package.yml'"));

    let expected = "This file represents the root package of the application\n";
    let actual = fs::read_to_string(
        "tests/fixtures/new_app/package.yml",
    ).unwrap_or_else(|_| panic!("Could not read file tests/fixtures/new_app/package.yml"));
    assert!(actual.contains(expected));


    common::teardown();
    common::delete_new_app();

    Ok(())
}
