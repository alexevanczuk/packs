#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, process::Command};
mod common;

#[test]
fn test_list_definitions_namespaced_experimental() -> Result<(), Box<dyn Error>>
{
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/app_with_namespaced_tests")
        .arg("--debug")
        .arg("--experimental-parser")
        .arg("list-definitions")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"::SomeModule::SomeOtherModule\" is defined at \"app/models/some_module/some_other_module.rb\""
        ))
        .stdout(predicate::str::contains(
            "\"::SomeModule::SomeOtherModule::SomeClass\" is defined at \"app/models/some_module/some_other_module/some_class.rb\""
        ))
        .stdout(predicate::str::contains(
            "\"::SomeModule::SomeOtherModule\" is defined at \"spec/models/some_module/some_other_module/some_class_spec.rb\""
        ).not());

    Ok(())
}
