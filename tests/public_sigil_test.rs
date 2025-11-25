use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;

mod common;

use regex::Regex;

#[test]
fn test_pack_with_public_api_exposed_via_sigil(
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/public_api_sigils")
        .arg("--debug")
        .arg("check")
        .output()?; // Capture the output

    // Convert stdout to a string for comparison
    let stdout_with_ansi = String::from_utf8_lossy(&output.stdout);

    // Regex to remove ANSI escape sequences
    let ansi_escape =
        Regex::new(r"\x1B\[([0-9]{1,2}(;[0-9]{1,2})?)?[m|K]").unwrap();
    let stdout = ansi_escape.replace_all(&stdout_with_ansi, "");

    // Define the expected output as a multiline string
    let expected_output = r#"1 violation(s) detected:
packs/foo/app/domain/foo/api.rb:7:8
Privacy violation: `::Bar::Api3` is private to `packs/bar`, but referenced from `packs/foo`


"#;

    // Verify the process fails
    assert!(!output.status.success());

    // Verify the output matches the expected output exactly
    assert_eq!(stdout, expected_output, "Unexpected output: {}", stdout);

    common::teardown();
    Ok(())
}

#[test]
// The intent of this test is to capture the fact that if we pass in a single file to the command,
// it will actually read the file to find the sigil. We normally don't do this when pks is run on the whole
// codebase to prevent a second pass of reading files, but its essential to the extension working correctly,
// since it only takes a subset of input files.
fn test_pack_with_public_api_exposed_via_sigil_with_single_fine_input(
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/public_api_sigils")
        .arg("--debug")
        .arg("check")
        .arg("packs/foo/app/domain/foo/api.rb")
        .output()?; // Capture the output

    // Convert stdout to a string for comparison
    let stdout_with_ansi = String::from_utf8_lossy(&output.stdout);

    // Regex to remove ANSI escape sequences
    let ansi_escape =
        Regex::new(r"\x1B\[([0-9]{1,2}(;[0-9]{1,2})?)?[m|K]").unwrap();
    let stdout = ansi_escape.replace_all(&stdout_with_ansi, "");

    // Define the expected output as a multiline string
    let expected_output = r#"1 violation(s) detected:
packs/foo/app/domain/foo/api.rb:7:8
Privacy violation: `::Bar::Api3` is private to `packs/bar`, but referenced from `packs/foo`


"#;

    // Verify the process fails
    assert!(!output.status.success());

    // Verify the output matches the expected output exactly
    assert_eq!(stdout, expected_output, "Unexpected output: {}", stdout);

    common::teardown();
    Ok(())
}

#[test]
fn test_pack_with_public_api_exposed_via_sigil_with_experimental_parser(
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(cargo_bin!("packs"))
        .arg("--project-root")
        .arg("tests/fixtures/public_api_sigils")
        .arg("--debug")
        .arg("--experimental-parser")
        .arg("check")
        .output()?; // Capture the output

    // Convert stdout to a string for comparison
    let stdout_with_ansi = String::from_utf8_lossy(&output.stdout);

    // Regex to remove ANSI escape sequences
    let ansi_escape =
        Regex::new(r"\x1B\[([0-9]{1,2}(;[0-9]{1,2})?)?[m|K]").unwrap();
    let stdout = ansi_escape.replace_all(&stdout_with_ansi, "");

    // Define the expected output as a multiline string
    let expected_output = r#"1 violation(s) detected:
packs/foo/app/domain/foo/api.rb:7:8
Privacy violation: `::Bar::Api3` is private to `packs/bar`, but referenced from `packs/foo`


"#;

    // Verify the process fails
    assert!(!output.status.success());

    // Verify the output matches the expected output exactly
    assert_eq!(stdout, expected_output, "Unexpected output: {}", stdout);

    common::teardown();
    Ok(())
}
