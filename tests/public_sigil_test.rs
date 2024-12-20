use assert_cmd::Command;
use predicates::prelude::*;
use std::error::Error;

mod common;

use regex::Regex;

#[test]
fn test_pack_with_public_api_exposed_via_sigil(
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::cargo_bin("packs")?
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
    let expected_output = r#"3 violation(s) detected:
packs/foo/app/domain/foo/api.rb:5:8
Privacy violation: `::Bar::Api` is private to `packs/bar`, but referenced from `packs/foo`

packs/foo/app/domain/foo/api.rb:6:8
Privacy violation: `::Bar::Api2` is private to `packs/bar`, but referenced from `packs/foo`

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
