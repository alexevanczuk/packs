use md5;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

pub(crate) fn file_content_digest(file: &PathBuf) -> String {
    let mut file_content = Vec::new();

    // Read the file content
    let mut file_handle = fs::File::open(file).unwrap_or_else(|_| panic!("Failed to open file {:?}", file));
    file_handle.read_to_end(&mut file_content).expect("Failed to read file");

    // Compute the MD5 digest
    let digest = md5::compute(&file_content);

    // Convert the digest to a hexadecimal string
    let hex_digest = format!("{:x}", digest);

    hex_digest
}

#[allow(dead_code)]
pub(crate) fn write_cache(absolute_root: &PathBuf, relative_path_to_file: &PathBuf) -> String {
    let digest_str = file_content_digest(file);

    // Create the cache JSON string
    let cache_json = format!(
        r#"{{
            "file_contents_digest": "{}",
            "unresolved_references": [
                {{
                    "constant_name": "::Bar",
                    "namespace_path": ["Foo"],
                    "relative_path": "packs/foo/app/services/foo.rb",
                    "source_location": {{
                        "line": 3,
                        "column": 6
                    }}
                }}
            ]
        }}"#,
        digest_str
    );

    // Convert the cache JSON string to bytes
    let cache_bytes = cache_json.as_bytes();

    // Compute the MD5 digest of the cache JSON bytes
    let cache_digest = md5::compute(cache_bytes);

    // Convert the cache digest to a hexadecimal string
    let cache_hex_digest = format!("{:x}", cache_digest);

    // Create the directory if it doesn't exist
    let cache_dir = PathBuf::from("tests/fixtures/simple_app/tmp/cache/packwerk/");
    fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");

    // Create the cache file path
    let cache_file = cache_dir.join(&cache_hex_digest);

    // Write the cache JSON bytes to the cache file
    fs::write(cache_file, cache_bytes).expect("Failed to write cache file");

    cache_hex_digest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_content_digest() {
        let file_path = "tests/fixtures/simple_app/packs/bar/app/services/bar.rb";
        let expected_digest = "f2af2fc657b71331ff3a8c39b48365eb";

        let digest = file_content_digest(&PathBuf::from(file_path));

        assert_eq!(digest, expected_digest);
    }

    #[test]
    fn test_write_cache() {
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

        write_cache(&PathBuf::from("tests/fixtures/simple_app/packs/foo/app/services/foo.rb"));
        let digest = md5::compute("packs/foo/app/services/foo.rb");
        let digest_str = format!("{:x}", digest);
        let cache_file = PathBuf::from("tests/fixtures/simple_app/tmp/cache/packwerk/").join(digest_str);
        let mut file = std::fs::File::open(&cache_file).unwrap_or_else(|_| panic!("Failed to open file {:?}", cache_file));
        let mut file_content = Vec::new();
        file.read_to_end(&mut file_content).expect("Failed to read file");
        let file_content_str = String::from_utf8_lossy(&file_content);
        assert_eq!(file_content_str, expected_cache_json);
    }
}
