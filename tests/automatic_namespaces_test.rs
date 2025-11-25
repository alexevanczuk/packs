use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};
mod common;

#[test]
fn test_automatic_namespaces_with_zeitwerk_parser() -> Result<(), Box<dyn Error>>
{
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/app_with_automatic_namespaces")
        .arg("--debug")
        .arg("list-definitions")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"::FooRecord\" is defined at \"packs/foo/app/models/foo_record.rb\""
        ))
        .stdout(predicate::str::contains(
            "\"::Foo::Creator\" is defined at \"packs/foo/app/services/creator.rb\""
        ));
    Ok(())
}

#[test]
fn test_automatic_namespaces_with_experimental_parser(
) -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/app_with_automatic_namespaces")
        .arg("--debug")
        // Experimental parser works without issues
        .arg("--experimental-parser")
        .arg("list-definitions")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"::FooRecord\" is defined at \"packs/foo/app/models/foo_record.rb\""
        ))
        .stdout(predicate::str::contains(
            "\"::Foo::Creator\" is defined at \"packs/foo/app/services/creator.rb\""
        ));
    Ok(())
}
