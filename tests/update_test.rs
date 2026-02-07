#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use serial_test::serial;
use std::{error::Error, path::Path, process::Command};
mod common;
use pretty_assertions::assert_eq;

#[test]
// This and the next test are run in serial because they both use the same fixtures.
#[serial]
fn test_update() -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains("package_todo.yml"));

    let package_todo_yml_filepath =
        Path::new("tests/fixtures/simple_app/packs/foo/package_todo.yml");
    let actual = std::fs::read_to_string(package_todo_yml_filepath)?;
    let expected = String::from(
        "\
# This file contains a list of dependencies that are not part of the long term plan for the
# 'packs/foo' package.
# We should generally work to reduce this list over time.
#
# You can regenerate this file using the following command:
#
# bin/packwerk update-todo
---
packs/bar:
  \"::Bar\":
    violations:
    - dependency
    - privacy
    files:
    - packs/foo/app/services/foo.rb
",
    );
    std::fs::remove_file(package_todo_yml_filepath)?;
    assert_eq!(expected, actual);

    common::teardown();

    Ok(())
}

#[test]
#[serial]
fn test_update_with_experimental_parser() -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("--debug")
        .arg("--experimental-parser")
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains("package_todo.yml"));

    let package_todo_yml_filepath =
        Path::new("tests/fixtures/simple_app/packs/foo/package_todo.yml");
    let actual = std::fs::read_to_string(package_todo_yml_filepath)?;
    let expected = String::from(
        "\
# This file contains a list of dependencies that are not part of the long term plan for the
# 'packs/foo' package.
# We should generally work to reduce this list over time.
#
# You can regenerate this file using the following command:
#
# bin/packwerk update-todo
---
packs/bar:
  \"::Bar\":
    violations:
    - dependency
    - privacy
    files:
    - packs/foo/app/services/foo.rb
",
    );
    std::fs::remove_file(package_todo_yml_filepath)?;
    assert_eq!(expected, actual);

    common::teardown();

    Ok(())
}

#[test]
fn test_update_with_stale_violations() -> Result<(), Box<dyn Error>> {
    common::set_up_fixtures();

    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/contains_stale_violations")
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 violation(s) removed"))
        .stdout(predicate::str::contains("1 file(s) modified"))
        .stdout(predicate::str::contains("1 file(s) deleted"));

    let package_todo_yml_filepath = Path::new(
        "tests/fixtures/contains_stale_violations/packs/foo/package_todo.yml",
    );
    let actual = std::fs::read_to_string(package_todo_yml_filepath)?;
    let expected = String::from(
        "\
# This file contains a list of dependencies that are not part of the long term plan for the
# 'packs/foo' package.
# We should generally work to reduce this list over time.
#
# You can regenerate this file using the following command:
#
# bin/packwerk update-todo
---
packs/bar:
  \"::Bar\":
    violations:
    - privacy
    files:
    - packs/foo/app/services/foo.rb
",
    );

    assert_eq!(expected, actual);

    let package_todo_yml_filepath = Path::new(
        "tests/fixtures/contains_stale_violations/packs/bar/package_todo.yml",
    );
    assert!(!package_todo_yml_filepath.exists());
    common::set_up_fixtures();

    Ok(())
}

#[test]
fn test_update_with_packs_first_app() -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_packs_first_app")
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 violation(s) added"))
        .stdout(predicate::str::contains("1 file(s) added"));

    let package_todo_yml_filepath = Path::new(
        "tests/fixtures/simple_packs_first_app/packs/foo/package_todo.yml",
    );
    let actual = std::fs::read_to_string(package_todo_yml_filepath)?;
    let expected = String::from(
        "\
# This file contains a list of dependencies that are not part of the long term plan for the
# 'packs/foo' package.
# We should generally work to reduce this list over time.
#
# You can regenerate this file using the following command:
#
# pks update
---
packs/bar:
  \"::Bar\":
    violations:
    - dependency
    - privacy
    files:
    - packs/foo/app/services/foo.rb
",
    );
    std::fs::remove_file(package_todo_yml_filepath)?;
    assert_eq!(expected, actual);

    common::teardown();

    Ok(())
}

#[test]
fn test_update_with_strict_violations() -> anyhow::Result<()> {
    let path = Path::new(
        "tests/fixtures/contains_strict_violations/packs/foo/package_todo.yml",
    );
    let _ignore = std::fs::remove_file(path);

    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/contains_strict_violations")
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "packs/foo cannot have privacy violations on packs/bar because strict mode is enabled for privacy violations in the enforcing pack's package.yml file",
        ))
        .stdout(predicate::str::contains("1 strict mode violation(s) detected."))
        .stdout(predicate::str::contains("No changes to package_todo.yml files."));

    assert!(
        !path.exists(),
        "todo should not be created for strict violations"
    );
    Ok(())
}

#[test]
#[serial]
fn test_update_with_file_arg() -> Result<(), Box<dyn Error>> {
    let package_todo_yml_filepath =
        Path::new("tests/fixtures/simple_app/packs/foo/package_todo.yml");
    let _ = std::fs::remove_file(package_todo_yml_filepath);

    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("update")
        .arg("packs/foo/app/services/foo.rb")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 violation(s) added"))
        .stdout(predicate::str::contains("1 file(s) added"));

    let actual = std::fs::read_to_string(package_todo_yml_filepath)?;
    let expected = String::from(
        "\
# This file contains a list of dependencies that are not part of the long term plan for the
# 'packs/foo' package.
# We should generally work to reduce this list over time.
#
# You can regenerate this file using the following command:
#
# bin/packwerk update-todo
---
packs/bar:
  \"::Bar\":
    violations:
    - dependency
    - privacy
    files:
    - packs/foo/app/services/foo.rb
",
    );
    std::fs::remove_file(package_todo_yml_filepath)?;
    assert_eq!(expected, actual);

    common::teardown();

    Ok(())
}

#[test]
#[serial]
fn test_update_with_file_arg_merges_into_existing() -> Result<(), Box<dyn Error>>
{
    let package_todo_yml_filepath =
        Path::new("tests/fixtures/simple_app/packs/foo/package_todo.yml");

    // Write a pre-existing package_todo.yml with a different violation
    let pre_existing = String::from(
        "\
# This file contains a list of dependencies that are not part of the long term plan for the
# 'packs/foo' package.
# We should generally work to reduce this list over time.
#
# You can regenerate this file using the following command:
#
# bin/packwerk update-todo
---
packs/baz:
  \"::Baz\":
    violations:
    - dependency
    files:
    - packs/foo/app/services/other.rb
",
    );
    std::fs::write(package_todo_yml_filepath, &pre_existing)?;

    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("update")
        .arg("packs/foo/app/services/foo.rb")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 violation(s) added"))
        .stdout(predicate::str::contains("1 file(s) modified"));

    let actual = std::fs::read_to_string(package_todo_yml_filepath)?;

    // The existing packs/baz violation should be preserved
    assert!(
        actual.contains("packs/baz:"),
        "existing violations should be preserved after merge"
    );
    assert!(
        actual.contains("\"::Baz\""),
        "existing constant should be preserved after merge"
    );
    // The new packs/bar violation should be added
    assert!(
        actual.contains("packs/bar:"),
        "new violations should be added by merge"
    );
    assert!(
        actual.contains("\"::Bar\""),
        "new constant should be added by merge"
    );

    std::fs::remove_file(package_todo_yml_filepath)?;
    common::teardown();

    Ok(())
}

#[test]
#[serial]
fn test_update_with_constant_filter() -> Result<(), Box<dyn Error>> {
    let package_todo_yml_filepath =
        Path::new("tests/fixtures/simple_app/packs/foo/package_todo.yml");
    let _ = std::fs::remove_file(package_todo_yml_filepath);

    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("update")
        .arg("packs/foo/app/services/foo.rb")
        .arg("--constant")
        .arg("::Bar")
        .arg("--violation-type")
        .arg("dependency")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 violation(s) added"));

    let actual = std::fs::read_to_string(package_todo_yml_filepath)?;
    let expected = String::from(
        "\
# This file contains a list of dependencies that are not part of the long term plan for the
# 'packs/foo' package.
# We should generally work to reduce this list over time.
#
# You can regenerate this file using the following command:
#
# bin/packwerk update-todo
---
packs/bar:
  \"::Bar\":
    violations:
    - dependency
    files:
    - packs/foo/app/services/foo.rb
",
    );
    std::fs::remove_file(package_todo_yml_filepath)?;
    assert_eq!(expected, actual);

    common::teardown();

    Ok(())
}

#[test]
#[serial]
fn test_update_with_pack_flag() -> Result<(), Box<dyn Error>> {
    let package_todo_yml_filepath =
        Path::new("tests/fixtures/simple_app/packs/foo/package_todo.yml");
    let _ = std::fs::remove_file(package_todo_yml_filepath);

    // Pass a single file but use --pack to expand to the whole pack
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("update")
        .arg("packs/foo/app/services/foo.rb")
        .arg("--pack")
        .assert()
        .success()
        .stdout(predicate::str::contains("violation(s) added"))
        .stdout(predicate::str::contains("file(s) added"));

    let actual = std::fs::read_to_string(package_todo_yml_filepath)?;
    assert!(
        actual.contains("packs/bar:"),
        "violations should be recorded"
    );
    assert!(
        actual.contains("\"::Bar\""),
        "constant should appear in package_todo"
    );

    std::fs::remove_file(package_todo_yml_filepath)?;
    common::teardown();

    Ok(())
}

#[test]
#[serial]
fn test_update_with_constant_filter_no_files() -> Result<(), Box<dyn Error>> {
    let package_todo_yml_filepath =
        Path::new("tests/fixtures/simple_app/packs/foo/package_todo.yml");
    let _ = std::fs::remove_file(package_todo_yml_filepath);

    // No file args, just --constant filter: scans all files but only merges matching violations
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("update")
        .arg("--constant")
        .arg("::Bar")
        .arg("--violation-type")
        .arg("dependency")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 violation(s) added"));

    let actual = std::fs::read_to_string(package_todo_yml_filepath)?;
    // Should only have the dependency violation, not privacy
    assert!(
        actual.contains("- dependency"),
        "dependency violation should be present"
    );
    assert!(
        !actual.contains("- privacy"),
        "privacy violation should NOT be present when filtered"
    );

    std::fs::remove_file(package_todo_yml_filepath)?;
    common::teardown();

    Ok(())
}

#[test]
fn test_update_with_pack_flag_requires_files() -> Result<(), Box<dyn Error>> {
    Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("update")
        .arg("--pack")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--pack requires at least one file argument",
        ));

    Ok(())
}
