use crate::packs::SourceLocation;
use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use super::caching::CachableFile;
use super::caching::Cache;

use super::parsing::Definition;
use super::parsing::Range;
use super::{ProcessedFile, UnresolvedReference};

pub struct PerFileCache {
    pub cache_dir: PathBuf,
}

impl Cache for PerFileCache {
    fn get(&self, absolute_root: &Path, path: &Path) -> Option<ProcessedFile> {
        let cachable_file =
            CachableFile::new(absolute_root, &self.cache_dir, path);
        let cache_entry = cache_entry_for_file(&cachable_file);
        if let Some(cache_entry) = cache_entry {
            let file_digests_match = cache_entry.file_contents_digest
                == cachable_file.file_contents_digest;

            if !file_digests_match {
                None
            } else {
                let processed_file = cache_entry.processed_file(path);
                Some(processed_file)
            }
        } else {
            None
        }
    }

    fn write(
        &self,
        cachable_file: &CachableFile,
        processed_file: &ProcessedFile,
    ) {
        let file_contents_digest =
            cachable_file.file_contents_digest.to_owned();
        let unresolved_references: Vec<ReferenceEntry> = processed_file
            .unresolved_references
            .iter()
            .map(|r| -> ReferenceEntry {
                ReferenceEntry {
                    constant_name: r.name.to_owned(),
                    namespace_path: r.namespace_path.to_owned(),
                    relative_path: cachable_file
                        .relative_path_string()
                        .to_owned(),
                    source_location: SourceLocation {
                        line: r.location.start_row,
                        column: r.location.start_col,
                    },
                }
            })
            .collect();

        let definitions = processed_file.definitions.clone();

        let cache_entry = &CacheEntry {
            file_contents_digest,
            unresolved_references,
            definitions,
        };

        let cache_data = serde_json::to_string(&cache_entry)
            .expect("Failed to serialize references");
        let mut file = File::create(&cachable_file.cache_file_path)
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to create cache file {:?}: {}",
                    cachable_file.cache_file_path, e
                )
            });

        file.write_all(cache_data.as_bytes())
            .expect("Failed to write cache file");
    }
}

fn cache_entry_for_file(file: &CachableFile) -> Option<CacheEntry> {
    let cache_file_path = &file.cache_file_path;

    if cache_file_path.exists() {
        Some(read_json_file(cache_file_path).unwrap_or_else(|_| {
            panic!("Failed to read cache file {:?}", cache_file_path)
        }))
    } else {
        None
    }
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheEntry {
    pub file_contents_digest: String,
    pub unresolved_references: Vec<ReferenceEntry>,
    #[serde(default)]
    pub definitions: Vec<Definition>,
}

impl CacheEntry {
    pub fn processed_file(self, absolute_path: &Path) -> ProcessedFile {
        let unresolved_references = self
            .unresolved_references
            .iter()
            .map(|reference| reference.to_unresolved_reference())
            .collect();

        ProcessedFile {
            unresolved_references,
            absolute_path: absolute_path.to_owned(),
            definitions: self.definitions,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct ReferenceEntry {
    constant_name: String,
    namespace_path: Vec<String>,
    relative_path: String,
    source_location: SourceLocation,
}

impl ReferenceEntry {
    fn to_unresolved_reference(&self) -> UnresolvedReference {
        UnresolvedReference {
            name: self.constant_name.to_owned(),
            namespace_path: self.namespace_path.to_owned(),
            location: Range {
                start_row: self.source_location.line,
                start_col: self.source_location.column,
                // The end row and end col can be improved here but we are limited
                // because the cache does not store this data.
                // Instead, we might just return a (resolved) Reference
                end_row: self.source_location.line,
                end_col: self.source_location.column + self.constant_name.len(),
            },
        }
    }
}

pub fn read_json_file(
    path: &PathBuf,
) -> Result<CacheEntry, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}

pub fn create_cache_dir_idempotently(cache_dir: &PathBuf) {
    std::fs::create_dir_all(cache_dir)
        .expect("Failed to create cache directory");
}

#[cfg(test)]
mod tests {
    use crate::packs::{self, configuration, file_utils::file_content_digest};

    use super::*;

    fn teardown() {
        packs::delete_cache(configuration::get(&PathBuf::from(
            "tests/fixtures/simple_app",
        )));
    }

    #[test]
    fn test_file_content_digest() {
        let file_path =
            "tests/fixtures/simple_app/packs/bar/app/services/bar.rb";
        let expected_digest = "305bc58696c2e664057b6751064cf2e3";

        let digest = file_content_digest(&PathBuf::from(file_path));

        assert_eq!(digest, expected_digest);

        teardown();
    }

    #[test]
    fn test_compatible_with_packwerk() {
        let contents: String = String::from(
            r#"{
    "file_contents_digest": "8f9efdcf2caa22fb7b1b4a8274e68d11",
    "unresolved_references": [
        {
            "constant_name": "Bar",
            "namespace_path": [
                "Foo",
                "Bar"
            ],
            "relative_path": "packs/foo/app/services/bar/foo.rb",
            "source_location": {
                "line": 8,
                "column": 22
            }
        }
    ]
}"#,
        );

        let actual_serialized =
            serde_json::from_str::<CacheEntry>(&contents).unwrap();
        let expected_serialized = CacheEntry {
            file_contents_digest: "8f9efdcf2caa22fb7b1b4a8274e68d11".to_owned(),
            unresolved_references: vec![ReferenceEntry {
                constant_name: "Bar".to_owned(),
                namespace_path: vec!["Foo".to_owned(), "Bar".to_owned()],
                relative_path: "packs/foo/app/services/bar/foo.rb".to_owned(),
                source_location: SourceLocation {
                    line: 8,
                    column: 22,
                },
            }],
            definitions: vec![],
        };

        assert_eq!(expected_serialized, actual_serialized);

        teardown();
    }

    #[test]
    fn test_compatible_with_alternate_parser() {
        let contents: String = String::from(
            r#"{
    "file_contents_digest": "8f9efdcf2caa22fb7b1b4a8274e68d11",
    "unresolved_references": [
        {
            "constant_name": "Bar",
            "namespace_path": [
                "Foo",
                "Bar"
            ],
            "relative_path": "packs/foo/app/services/bar/foo.rb",
            "source_location": {
                "line": 8,
                "column": 22
            }
        }
    ],
    "definitions": []
}"#,
        );

        let actual_serialized =
            serde_json::from_str::<CacheEntry>(&contents).unwrap();
        let expected_serialized = CacheEntry {
            file_contents_digest: "8f9efdcf2caa22fb7b1b4a8274e68d11".to_owned(),
            unresolved_references: vec![ReferenceEntry {
                constant_name: "Bar".to_owned(),
                namespace_path: vec!["Foo".to_owned(), "Bar".to_owned()],
                relative_path: "packs/foo/app/services/bar/foo.rb".to_owned(),
                source_location: SourceLocation {
                    line: 8,
                    column: 22,
                },
            }],
            definitions: vec![],
        };

        assert_eq!(expected_serialized, actual_serialized);

        teardown();
    }
}
