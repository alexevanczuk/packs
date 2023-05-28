use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{error::Error, io::Read, path::PathBuf, process::Command};

#[test]
fn test_generate_cache() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("pks")?
        .arg("--project-root")
        .arg("tests/fixtures/simple_app")
        .arg("generate-cache")
        .arg("packs/bar/app/services/bar.rb")
        .arg("packs/foo/app/services/foo.rb")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Cache was generated for files packs/bar/app/services/bar.rb and packs/foo/app/services/foo.rb\n",
        ))
        .stdout(predicate::str::contains(
            "The file content digests are {\"packs/bar/app/services/bar.rb\": \"f2af2fc657b71331ff3a8c39b48365eb\", \"packs/foo/app/services/foo.rb\": \"4be8effd7ac57323adcb53d0cf0ce789\"}",
        ));

    let expected_cache_json = r#"{
            "file_contents_digest": "4be8effd7ac57323adcb53d0cf0ce789",
            "unresolved_references": [
                {
                    "constant_name": "::Bar",
                    "namespace_path": ["Foo"],
                    "relative_path": "packs/foo/app/services/foo.rb",
                    "source_location": {
                        "line": 3,
                        "column": 6
                    }
                }
            ]
        }"#;

    let digest = md5::compute("packs/foo/app/services/foo.rb");
    let digest_str = format!("{:x}", digest);
    let cache_file = PathBuf::from("tests/fixtures/simple_app/tmp/cache/packwerk/").join(digest_str);
    let mut file = std::fs::File::open(&cache_file).unwrap_or_else(|_| panic!("Failed to open file {:?}", cache_file));
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content).expect("Failed to read file");
    let file_content_str = String::from_utf8_lossy(&file_content);
    assert_eq!(file_content_str, expected_cache_json);
    Ok(())
}
