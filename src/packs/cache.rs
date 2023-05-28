use crate::packs::parser::extract_from_path;
use md5;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use super::parser::Reference;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct CacheEntry {
    file_contents_digest: String,
    unresolved_references: Vec<ReferenceEntry>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct SourceLocation {
    line: usize,
    column: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ReferenceEntry {
    constant_name: String,
    namespace_path: Vec<String>,
    relative_path: String,
    source_location: SourceLocation,
}

// Used for tests, for now!
#[allow(dead_code)]
fn read_json_file(
    path: &PathBuf,
) -> Result<CacheEntry, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
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

fn references_to_cache_entry(
    _references: Vec<Reference>,
    file_contents_digest: String,
) -> CacheEntry {
    let unresolved_references: Vec<ReferenceEntry> = vec![];
    CacheEntry {
        file_contents_digest,
        unresolved_references,
    }
}
#[allow(dead_code)]
pub(crate) fn write_cache(absolute_root: &Path, relative_path_to_file: &Path) {
    let references =
        extract_from_path(absolute_root.join(relative_path_to_file));

    let cache_dir = absolute_root.join("tmp/cache/packwerk");
    std::fs::create_dir_all(&cache_dir)
        .expect("Failed to create cache directory");

    let file_digest = md5::compute(relative_path_to_file.to_str().unwrap());
    let file_digest_str = format!("{:x}", file_digest);
    let cache_file_path = cache_dir.join(&file_digest_str);
    let cache_entry = references_to_cache_entry(references, file_digest_str);

    let cache_data = serde_json::to_string(&cache_entry)
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
        let expected = CacheEntry {
            file_contents_digest: String::from(
                // This is the MD5 digest of the string literal "packs/foo/app/services/foo.rb"
                "061bf98e1706eac5af59c4b1a770fc7e",
            ),
            unresolved_references: vec![ReferenceEntry {
                constant_name: String::from("::Bar"),
                namespace_path: vec![String::from("Foo")],
                relative_path: String::from("packs/foo/app/services/foo.rb"),
                source_location: SourceLocation { line: 3, column: 6 },
            }],
        };

        write_cache(
            &PathBuf::from("tests/fixtures/simple_app"),
            &PathBuf::from("packs/foo/app/services/foo.rb"),
        );

        let cache_file = PathBuf::from("tests/fixtures/simple_app/tmp/cache/packwerk/061bf98e1706eac5af59c4b1a770fc7e");
        let actual = read_json_file(&cache_file).unwrap();
        assert_eq!(actual, expected);
    }
}
