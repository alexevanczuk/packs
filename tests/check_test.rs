use assert_cmd::Command;
use predicates::prelude::*;
use std::{error::Error, fs};

mod common;

#[test]
fn test_check() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
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
fn test_check_with_single_file() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("check")
        .arg("packs/foo/app/services/foo.rb")
        .assert()
        .failure()
        .stdout(predicate::str::contains("2 violation(s) detected:"))
        .stdout(predicate::str::contains("packs/foo/app/services/foo.rb:3:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."))
        .stdout(predicate::str::contains("packs/foo/app/services/foo.rb:3:4\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_with_single_file_experimental_parser(
) -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("--experimental-parser")
        .arg("check")
        .arg("packs/foo/app/services/foo.rb")
        .assert()
        .failure()
        .stdout(predicate::str::contains("2 violation(s) detected:"))
        .stdout(predicate::str::contains("packs/foo/app/services/foo.rb:3:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."))
        .stdout(predicate::str::contains("packs/foo/app/services/foo.rb:3:4\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"));

    common::teardown();
    Ok(())
}

#[test]
fn test_check_with_package_todo_file() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")?
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
    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg("tests/fixtures/contains_package_todo")
        .arg("--debug")
        .arg("check")
        .arg("--ignore-recorded-violations")
        .assert()
        .failure()
        .stdout(predicate::str::contains("2 violation(s) detected:"))
        .stdout(predicate::str::contains("packs/foo/app/services/foo.rb:3:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."))
        .stdout(predicate::str::contains("packs/foo/app/services/other_foo.rb:3:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."));

    common::teardown();

    Ok(())
}

#[test]
fn test_check_with_experimental_parser() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")
        .unwrap()
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--experimental-parser")
        .arg("--debug")
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
fn test_check_with_stale_violations() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")
        .unwrap()
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
    Command::cargo_bin("packs")
        .unwrap()
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
fn test_check_without_stale_violations() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("packs")
        .unwrap()
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
    Command::cargo_bin("packs")
        .unwrap()
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

    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg(project_root)
        .arg("--debug")
        .arg("check-contents")
        .arg(relative_path)
        .write_stdin(format!("\n\n\n{}", foo_rb_contents))
        .assert()
        .failure()
        .stdout(predicate::str::contains("2 violation(s) detected:"))
        .stdout(predicate::str::contains("packs/foo/app/services/foo.rb:6:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."))
        .stdout(predicate::str::contains("packs/foo/app/services/foo.rb:6:4\nPrivacy violation: `::Bar` is private to `packs/bar`, but referenced from `packs/foo`"));

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

    Command::cargo_bin("packs")?
        .arg("--project-root")
        .arg(project_root)
        .arg("--debug")
        .arg("check-contents")
        .arg("--ignore-recorded-violations")
        .arg(relative_path)
        .write_stdin(format!("\n\n\n{}", foo_rb_contents))
        .assert()
        .failure()
        .stdout(predicate::str::contains("1 violation(s) detected:"))
        .stdout(predicate::str::contains("packs/foo/app/services/foo.rb:6:4\nDependency violation: `::Bar` belongs to `packs/bar`, but `packs/foo/package.yml` does not specify a dependency on `packs/bar`."));

    common::teardown();
    Ok(())
}
