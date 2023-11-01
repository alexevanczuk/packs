use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, fs, process::Command};
mod common;

#[test]
fn test_check_unnecessary_dependencies() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/app_with_dependency_cycles")
        .arg("--debug")
        .arg("check-unnecessary-dependencies")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "packs/bar depends on packs/foo but does not use it",
        ))
        .stdout(predicate::str::contains(
            "packs/foo depends on packs/bar but does not use it",
        ));
    Ok(())
}

#[test]
fn test_auto_correct_unnecessary_dependencies() -> Result<(), Box<dyn Error>> {
    let foo_package_path = "tests/fixtures/app_with_unnecessary_dependencies/packs/foo/package.yml";
    let starting_foo_package_yml = [
        "enforce_dependencies: true",
        "enforce_privacy: true",
        "dependencies:",
        "  - packs/bar",
        "  - packs/baz",
    ];
    fs::write(foo_package_path, starting_foo_package_yml.join("\n")).unwrap();
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/app_with_unnecessary_dependencies")
        .arg("--debug")
        .arg("check-unnecessary-dependencies")
        .arg("--auto-correct")
        .assert()
        .success();

    let expected_autocorrect = [
        "enforce_dependencies: true",
        "enforce_privacy: true",
        "dependencies:",
        "  - packs/bar\n",
    ]
    .join("\n");
    let after_autocorrect = fs::read_to_string(foo_package_path).unwrap();
    assert_eq!(after_autocorrect, expected_autocorrect);

    Ok(())
}

#[test]
fn test_check_unnecessary_dependencies_no_issue() -> Result<(), Box<dyn Error>>
{
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("check-unnecessary-dependencies")
        .assert()
        .success();
    Ok(())
}
