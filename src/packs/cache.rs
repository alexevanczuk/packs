use crate::packs::parser::extract_from_path;
use md5;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct CacheEntry {
    file_contents_digest: String,
    unresolved_references: Vec<ReferenceEntry>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Location {
    line: usize,
    column: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ReferenceEntry {
    constant_name: String,
    namespace_path: Vec<String>,
    relative_path: String,
    source_location: Location,
}
pub(crate) fn file_content_digest(file: &PathBuf) -> String {
    let mut file_content = Vec::new();

    // Read the file content
    let mut file_handle = fs::File::open(file)
        .unwrap_or_else(|_| panic!("Failed to open file {:?}", file));
    file_handle
        .read_to_end(&mut file_content)
        .expect("Failed to read file");

    // Compute the MD5 digest
    format!("{:x}", md5::compute(&file_content))
}

#[allow(dead_code)]
pub(crate) fn write_cache(absolute_root: &Path, relative_path_to_file: &Path) {
    let references =
        extract_from_path(absolute_root.join(relative_path_to_file));
    let cache_dir = absolute_root.join("tmp/cache/packwerk");
    std::fs::create_dir_all(&cache_dir)
        .expect("Failed to create cache directory");

    let file_digest = md5::compute(relative_path_to_file.to_str().unwrap());
    let file_name = format!("{:x}", file_digest);

    let cache_file_path = cache_dir.join(file_name);

    let cache_data = serde_json::to_string(&references)
        .expect("Failed to serialize references");
    let mut file =
        File::create(cache_file_path).expect("Failed to create cache file");
    file.write_all(cache_data.as_bytes())
        .expect("Failed to write cache file");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_content_digest() {
        let file_path =
            "tests/fixtures/simple_app/packs/bar/app/services/bar.rb";
        let expected_digest = "f2af2fc657b71331ff3a8c39b48365eb";

        let digest = file_content_digest(&PathBuf::from(file_path));

        assert_eq!(digest, expected_digest);
    }

    #[test]
    fn test_write_cache() {
        let expected_cache_json = CacheEntry {
            file_contents_digest: String::from(
                "4be8effd7ac57323adcb53d0cf0ce789",
            ),
            unresolved_references: vec![ReferenceEntry {
                constant_name: String::from("::Bar"),
                namespace_path: vec![String::from("Foo")],
                relative_path: String::from("packs/foo/app/services/foo.rb"),
                source_location: Location { line: 3, column: 6 },
            }],
        };

        write_cache(
            &PathBuf::from("tests/fixtures/simple_app"),
            &PathBuf::from("packs/foo/app/services/foo.rb"),
        );

        // This is the MD5 digest of "packs/foo/app/services/foo.rb"
        let digest_str = "061bf98e1706eac5af59c4b1a770fc7e";
        let cache_file = PathBuf::from("tests/fixtures/simple_app/tmp/cache/packwerk/061bf98e1706eac5af59c4b1a770fc7e");
        let mut file = std::fs::File::open(&cache_file)
            .unwrap_or_else(|_| panic!("Failed to open file {:?}", cache_file));
        let mut file_content = Vec::new();
        file.read_to_end(&mut file_content)
            .expect("Failed to read file");

        let file_content_str = String::from_utf8_lossy(&file_content);
        assert_eq!(expected_cache_json, file_content_str);
    }
}
