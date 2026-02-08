#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::{error::Error, fs};

mod common;

pub fn stripped_output(output: Vec<u8>) -> String {
    String::from_utf8_lossy(&strip_ansi_escapes::strip(output)).to_string()
}

#[test]
fn test_check() -> Result<(), Box<dyn Error>> {
    let output = Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("check")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let stripped_output = stripped_output(output);

    assert!(stripped_output.contains("2 violation(s) detected:"));
    assert!(stripped_output.contains("packs/foo/app/services/foo.rb:3:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."));
    assert!(stripped_output.contains("packs/foo/app/services/foo.rb:3:4\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_enforce_privacy_disabled() -> Result<(), Box<dyn Error>> {
    let output = Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("--disable-enforce-privacy")
        .arg("check")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let stripped_output = stripped_output(output);

    assert!(stripped_output.contains("1 violation(s) detected:"));
    assert!(stripped_output.contains("packs/foo/app/services/foo.rb:3:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_enforce_dependency_disabled() -> Result<(), Box<dyn Error>> {
    let output = Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("--disable-enforce-dependencies")
        .arg("check")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let stripped_output = stripped_output(output);

    assert!(stripped_output.contains("1 violation(s) detected:"));
    assert!(stripped_output.contains("packs/foo/app/services/foo.rb:3:4\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_with_single_file() -> Result<(), Box<dyn Error>> {
    let output = Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("check")
        .arg("packs/foo/app/services/foo.rb")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let stripped_output = stripped_output(output);

    assert!(stripped_output.contains("2 violation(s) detected:"));
    assert!(stripped_output.contains("packs/foo/app/services/foo.rb:3:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."));
    assert!(stripped_output.contains("packs/foo/app/services/foo.rb:3:4\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_with_single_file_experimental_parser(
) -> Result<(), Box<dyn Error>> {
    let output = Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("--experimental-parser")
        .arg("check")
        .arg("packs/foo/app/services/foo.rb")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let stripped_output = stripped_output(output);

    assert!(stripped_output.contains("2 violation(s) detected:"));
    assert!(stripped_output.contains("packs/foo/app/services/foo.rb:3:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."));
    assert!(stripped_output.contains("packs/foo/app/services/foo.rb:3:4\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_with_package_todo_file() -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/contains_package_todo")
        .arg("--debug")
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("No violations detected!"));

    common::teardown();

    Ok(())
}

#[test]
fn test_check_with_package_todo_file_ignoring_recorded_violations(
) -> Result<(), Box<dyn Error>> {
    let output = Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/contains_package_todo")
        .arg("--debug")
        .arg("check")
        .arg("--ignore-recorded-violations")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let stripped_output = stripped_output(output);
    assert!(stripped_output.contains("2 violation(s) detected:"));
    assert!(stripped_output.contains("packs/foo/app/services/foo.rb:3:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."));
    assert!(stripped_output.contains("packs/foo/app/services/other_foo.rb:3:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."));

    common::teardown();

    Ok(())
}

#[test]
fn test_check_with_experimental_parser() -> Result<(), Box<dyn Error>> {
    let output = Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--experimental-parser")
        .arg("--debug")
        .arg("check")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let stripped_output = stripped_output(output);

    assert!(stripped_output.contains("2 violation(s) detected:"));
    assert!(stripped_output.contains("packs/foo/app/services/foo.rb:3:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."));
    assert!(stripped_output.contains("packs/foo/app/services/foo.rb:3:4\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_with_stale_violations() -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/contains_stale_violations")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "There were stale violations found, please run `packs update`",
        ));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_with_stale_violations_when_file_no_longer_exists(
) -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/contains_stale_violations_no_file")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "There were stale violations found, please run `packs update`",
        ));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_with_relationship_violations() -> Result<(), Box<dyn Error>> {
    // Tests that associations with explicit class_name (using .name) are correctly resolved
    // The fixture has:
    //   has_many :censuses       -> Census
    //   has_many :tacos          -> Taco
    //   belongs_to :my_widget, class_name: Census.name  -> Census (NOT MyWidget)
    // Plus a direct reference to Census in the class_name argument itself
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/app_with_rails_relationships")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains("4 violation(s) detected:"))
        .stdout(predicate::str::contains("Privacy violation: `::Taco` is private to `packs/baz`, but referenced from `packs/bar`"))
        .stdout(predicate::str::contains("Privacy violation: `::Census` is private to `packs/baz`, but referenced from `packs/bar`"));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_without_stale_violations() -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/contains_package_todo")
        .arg("check")
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "There were stale violations found, please run `packs update`",
            )
            .not(),
        );

    common::teardown();
    Ok(())
}

#[test]
fn test_check_with_strict_mode() -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/uses_strict_mode")
        .arg("check")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "packs/foo cannot have privacy violations on packs/bar because strict mode is enabled for privacy violations in the enforcing pack's package.yml file",
        ))
        .stdout(predicate::str::contains(
            "packs/foo cannot have dependency violations on packs/bar because strict mode is enabled for dependency violations in the enforcing pack's package.yml file",
        ));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_contents() -> Result<(), Box<dyn Error>> {
    let project_root = "tests/fixtures/simple_app";
    let relative_path = "packs/foo/app/services/foo.rb";
    let foo_rb_contents =
        fs::read_to_string(format!("{}/{}", project_root, relative_path))?;

    let output = Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg(project_root)
        .arg("--debug")
        .arg("check-contents")
        .arg(relative_path)
        .write_stdin(format!("\n\n\n{}", foo_rb_contents))
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let stripped_output = stripped_output(output);

    assert!(stripped_output.contains("2 violation(s) detected:"));
    assert!(stripped_output.contains("packs/foo/app/services/foo.rb:6:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."));
    assert!(stripped_output.contains("packs/foo/app/services/foo.rb:6:4\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_contents_ignoring_recorded_violations(
) -> Result<(), Box<dyn Error>> {
    let project_root = "tests/fixtures/contains_package_todo";
    let relative_path = "packs/foo/app/services/foo.rb";
    let foo_rb_contents =
        fs::read_to_string(format!("{}/{}", project_root, relative_path))?;

    let output = Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg(project_root)
        .arg("--debug")
        .arg("check-contents")
        .arg("--ignore-recorded-violations")
        .arg(relative_path)
        .write_stdin(format!("\n\n\n{}", foo_rb_contents))
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let stripped_output = stripped_output(output);
    assert!(stripped_output.contains("1 violation(s) detected:"));
    assert!(stripped_output.contains("packs/foo/app/services/foo.rb:6:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_json_output() -> Result<(), Box<dyn Error>> {
    let output = Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("check")
        .arg("--json")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output)?;

    assert_eq!(json["status"], "failure");

    let violations = json["violations"].as_array().unwrap();
    assert_eq!(violations.len(), 2);

    // Violations are sorted by message, dependency comes before privacy
    let dep = &violations[0];
    assert_eq!(dep["file"], "packs/foo/app/services/foo.rb");
    assert_eq!(dep["line"], 3);
    assert_eq!(dep["column"], 4);
    assert_eq!(dep["violation_type"], "dependency");
    assert_eq!(dep["constant_name"], "::Bar");
    assert_eq!(dep["referencing_pack_name"], "packs/foo");
    assert_eq!(dep["defining_pack_name"], "packs/bar");
    assert_eq!(dep["strict"], false);
    assert!(dep["message"]
        .as_str()
        .unwrap()
        .contains("Dependency violation"));
    // No ANSI escape codes in JSON message
    assert!(!dep["message"].as_str().unwrap().contains("\x1b"));

    let priv_v = &violations[1];
    assert_eq!(priv_v["violation_type"], "privacy");
    assert!(priv_v["message"]
        .as_str()
        .unwrap()
        .contains("Privacy violation"));

    assert!(json["stale_violations"].as_array().unwrap().is_empty());
    assert!(json["strict_mode_violations"]
        .as_array()
        .unwrap()
        .is_empty());

    common::teardown();
    Ok(())
}

#[test]
fn test_check_json_no_violations() -> Result<(), Box<dyn Error>> {
    let output = Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/contains_package_todo")
        .arg("check")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output)?;

    assert_eq!(json["status"], "success");
    assert!(json["violations"].as_array().unwrap().is_empty());

    common::teardown();
    Ok(())
}

#[test]
fn test_check_contents_json() -> Result<(), Box<dyn Error>> {
    let project_root = "tests/fixtures/simple_app";
    let relative_path = "packs/foo/app/services/foo.rb";
    let foo_rb_contents =
        fs::read_to_string(format!("{}/{}", project_root, relative_path))?;

    let output = Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg(project_root)
        .arg("check-contents")
        .arg("--json")
        .arg(relative_path)
        .write_stdin(foo_rb_contents)
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let json: Value = serde_json::from_slice(&output)?;

    assert_eq!(json["status"], "failure");
    let violations = json["violations"].as_array().unwrap();
    assert_eq!(violations.len(), 2);
    assert_eq!(violations[0]["file"], "packs/foo/app/services/foo.rb");

    common::teardown();
    Ok(())
}
