use assert_cmd::Command;
#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use std::{error::Error, fs};

mod common;

#[test]
fn test_create() -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("create")
        .arg("packs/foobar")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Successfully created `packs/foobar`!",
        ));

    let expected = "enforce_dependencies: true\n";
    let actual = fs::read_to_string(
        "tests/fixtures/simple_app/packs/foobar/package.yml",
    ).unwrap_or_else(|_| panic!("Could not read file tests/fixtures/simple_app/packs/foobar/package.yml"));
    assert_eq!(expected, actual);

    let expected_readme = String::from("\
Welcome to `packs/foobar`!

If you're the author, please consider replacing this file with a README.md, which may contain:
- What your pack is and does
- How you expect people to use your pack
- Example usage of your pack's public API and where to find it
- Limitations, risks, and important considerations of usage
- How to get in touch with eng and other stakeholders for questions or issues pertaining to this pack
- What SLAs/SLOs (service level agreements/objectives), if any, your package provides
- When in doubt, keep it simple
- Anything else you may want to include!

README.md should change as your public API changes.

See https://github.com/rubyatscale/packs#readme for more info!");

    let actual_readme =
        fs::read_to_string("tests/fixtures/simple_app/packs/foobar/README.md").unwrap_or_else(|e| {
            panic!("Could not read file tests/fixtures/simple_app/packs/foobar/README.md: {}", e)
        });

    assert_eq!(expected_readme, actual_readme);

    common::teardown();
    common::delete_foobar();

    Ok(())
}

#[test]
fn test_create_already_exists() -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_packs_first_app")
        .arg("create")
        .arg("packs/foo")
        .assert()
        .success()
        .stdout(predicate::str::contains("`packs/foo` already exists!"));

    let expected = String::from(
        "\
enforce_dependencies: true
enforce_privacy: true
dependencies:
- packs/baz
",
    );

    let actual =
        fs::read_to_string("tests/fixtures/simple_app/packs/foo/package.yml")?;
    assert_eq!(expected, actual);
    common::teardown();
    Ok(())
}
