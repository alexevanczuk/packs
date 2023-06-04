use crate::packs::parser::extract_from_ruby_path;
use crate::packs::parser::UnresolvedReference;
use crate::packs::Configuration;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct CacheEntry {
    file_contents_digest: String,
    unresolved_references: Vec<ReferenceEntry>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
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
    references: Vec<UnresolvedReference>,
    file_contents_digest: String,
    relative_path: String,
) -> CacheEntry {
    let unresolved_references: Vec<ReferenceEntry> = references
        .iter()
        .map(|r| -> ReferenceEntry {
            ReferenceEntry {
                constant_name: r.name.to_owned(),
                namespace_path: r.namespace_path.to_owned(),
                relative_path: relative_path.to_owned(),
                source_location: SourceLocation {
                    line: r.location.start_row,
                    column: r.location.start_col,
                },
            }
        })
        .collect();
    CacheEntry {
        file_contents_digest,
        unresolved_references,
    }
}

fn write_cache(
    absolute_root: &Path,
    relative_path_to_file: &Path,
    references: Vec<UnresolvedReference>,
) {
    let absolute_path = absolute_root.join(relative_path_to_file);

    let cache_dir = absolute_root.join("tmp/cache/packwerk");
    std::fs::create_dir_all(&cache_dir)
        .expect("Failed to create cache directory");

    let file_digest = md5::compute(relative_path_to_file.to_str().unwrap());
    let file_digest_str = env::var("CACHE_VERIFICATION")
        .map(|_| format!("{:x}-experimental", file_digest))
        .unwrap_or_else(|_| format!("{:x}", file_digest));

    let cache_file_path = cache_dir.join(file_digest_str);
    let cache_entry = references_to_cache_entry(
        references,
        file_content_digest(&absolute_path),
        relative_path_to_file
            .to_str()
            .expect("Could not convert cache_file_path to string")
            .to_string(),
    );

    let cache_data = serde_json::to_string(&cache_entry)
        .expect("Failed to serialize references");
    let mut file =
        File::create(cache_file_path).expect("Failed to create cache file");
    file.write_all(cache_data.as_bytes())
        .expect("Failed to write cache file");
}

pub(crate) fn write_cache_for_files(
    files: Vec<String>,
    configuration: Configuration,
) {
    let absolute_root_path = configuration.absolute_root.as_path();
    let absolute_paths: HashSet<PathBuf> = configuration.intersect_files(files);

    absolute_paths.par_iter().for_each(|path| {
        let relative_path = path.strip_prefix(absolute_root_path).unwrap();
        let references = extract_from_ruby_path(path);

        write_cache(absolute_root_path, relative_path, references);
    })
}

#[cfg(test)]
mod tests {
    use crate::packs::configuration;

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
    fn test_write_cache_for_files() {
        let expected = CacheEntry {
            file_contents_digest: String::from(
                // This is the MD5 digest of the contents of "packs/foo/app/services/foo.rb"
                // i.e. in ruby, it's:
                // Digest::MD5.hexdigest(File.read('tests/fixtures/simple_app/packs/foo/app/services/foo.rb'))
                "4be8effd7ac57323adcb53d0cf0ce789",
            ),
            unresolved_references: vec![
                ReferenceEntry {
                    constant_name: String::from("::Foo"),
                    namespace_path: vec![String::from("Foo")],
                    relative_path: String::from(
                        "packs/foo/app/services/foo.rb",
                    ),
                    source_location: SourceLocation { line: 1, column: 7 },
                },
                ReferenceEntry {
                    constant_name: String::from("Bar"),
                    namespace_path: vec![String::from("Foo")],
                    relative_path: String::from(
                        "packs/foo/app/services/foo.rb",
                    ),
                    source_location: SourceLocation { line: 3, column: 4 },
                },
            ],
        };

        write_cache_for_files(
            vec![String::from("packs/foo/app/services/foo.rb")],
            configuration::get(PathBuf::from("tests/fixtures/simple_app")),
        );

        let cache_file = PathBuf::from("tests/fixtures/simple_app/tmp/cache/packwerk/061bf98e1706eac5af59c4b1a770fc7e-experimental");
        let actual = read_json_file(&cache_file).unwrap();
        assert_eq!(actual, expected);
    }
}
