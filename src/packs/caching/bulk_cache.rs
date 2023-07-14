use crate::packs::file_utils::file_content_digest;
use crate::packs::ProcessedFile;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use super::Cache;
use super::CacheResult;
use super::EmptyCacheEntry;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BulkCacheEntry {
    pub processed_file_by_file_content_digest: HashMap<String, ProcessedFile>,
}

pub struct BulkCache {
    pub cache_dir: PathBuf,
    pub contents: BulkCacheEntry,
}

impl Cache for BulkCache {
    fn get(&self, path: &Path) -> CacheResult {
        let empty_cache_entry = EmptyCacheEntry::new(&self.cache_dir, path);
        let maybe_processed_file = self
            .contents
            .processed_file_by_file_content_digest
            .get(&empty_cache_entry.file_contents_digest);
        if let Some(processed_file) = maybe_processed_file {
            CacheResult::Processed(processed_file.to_owned())
        } else {
            CacheResult::Miss(empty_cache_entry)
        }
    }

    fn write(
        &self,
        _empty_cache_entry: &EmptyCacheEntry,
        _processed_file: &ProcessedFile,
    ) {
        // Do nothing
    }

    fn setup(&mut self, _cache_dir: &Path) {
        let cache_file_path = self.cache_dir.join("bulk_cache.json");
        let cache_file = File::open(cache_file_path).unwrap();
        let cache_entry: BulkCacheEntry = serde_json::from_reader(cache_file)
            .unwrap_or_else(|_| panic!("Failed to read bulk cache file"));

        self.contents = cache_entry;
    }

    fn write_all(&self, processed_files: &[ProcessedFile]) {
        // Do the opposite of setup
        let cache_file_path = self.cache_dir.join("bulk_cache.json");
        let cache_file = File::create(cache_file_path).unwrap();

        let cache_entry = BulkCacheEntry {
            processed_file_by_file_content_digest: processed_files
                .iter()
                .map(|processed_file| {
                    let md5 =
                        file_content_digest(&processed_file.absolute_path);
                    (md5.to_owned(), processed_file.to_owned())
                })
                .collect(),
        };

        // Write cache_entry
        serde_json::to_writer(cache_file, &cache_entry).unwrap();
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::packs::{self, configuration, file_utils::file_content_digest};

//     use super::*;

//     fn teardown() {
//         packs::delete_cache(configuration::get(&PathBuf::from(
//             "tests/fixtures/simple_app",
//         )));
//     }

//     #[test]
//     fn test_file_content_digest() {
//         let file_path =
//             "tests/fixtures/simple_app/packs/bar/app/services/bar.rb";
//         let expected_digest = "305bc58696c2e664057b6751064cf2e3";

//         let digest = file_content_digest(&PathBuf::from(file_path));

//         assert_eq!(digest, expected_digest);

//         teardown();
//     }

//     #[test]
//     fn test_compatible_with_packwerk() {
//         let contents: String = String::from(
//             r#"{
//     "file_contents_digest": "8f9efdcf2caa22fb7b1b4a8274e68d11",
//     "unresolved_references": [
//         {
//             "constant_name": "Bar",
//             "namespace_path": [
//                 "Foo",
//                 "Bar"
//             ],
//             "relative_path": "packs/foo/app/services/bar/foo.rb",
//             "source_location": {
//                 "line": 8,
//                 "column": 22
//             }
//         }
//     ]
// }"#,
//         );

//         let actual_serialized =
//             serde_json::from_str::<CacheEntry>(&contents).unwrap();
//         let expected_serialized = CacheEntry {
//             file_contents_digest: "8f9efdcf2caa22fb7b1b4a8274e68d11".to_owned(),
//             unresolved_references: vec![ReferenceEntry {
//                 constant_name: "Bar".to_owned(),
//                 namespace_path: vec!["Foo".to_owned(), "Bar".to_owned()],
//                 relative_path: "packs/foo/app/services/bar/foo.rb".to_owned(),
//                 source_location: SourceLocation {
//                     line: 8,
//                     column: 22,
//                 },
//             }],
//             definitions: vec![],
//         };

//         assert_eq!(expected_serialized, actual_serialized);

//         teardown();
//     }

//     #[test]
//     fn test_compatible_with_alternate_parser() {
//         let contents: String = String::from(
//             r#"{
//     "file_contents_digest": "8f9efdcf2caa22fb7b1b4a8274e68d11",
//     "unresolved_references": [
//         {
//             "constant_name": "Bar",
//             "namespace_path": [
//                 "Foo",
//                 "Bar"
//             ],
//             "relative_path": "packs/foo/app/services/bar/foo.rb",
//             "source_location": {
//                 "line": 8,
//                 "column": 22
//             }
//         }
//     ],
//     "definitions": []
// }"#,
//         );

//         let actual_serialized =
//             serde_json::from_str::<CacheEntry>(&contents).unwrap();
//         let expected_serialized = CacheEntry {
//             file_contents_digest: "8f9efdcf2caa22fb7b1b4a8274e68d11".to_owned(),
//             unresolved_references: vec![ReferenceEntry {
//                 constant_name: "Bar".to_owned(),
//                 namespace_path: vec!["Foo".to_owned(), "Bar".to_owned()],
//                 relative_path: "packs/foo/app/services/bar/foo.rb".to_owned(),
//                 source_location: SourceLocation {
//                     line: 8,
//                     column: 22,
//                 },
//             }],
//             definitions: vec![],
//         };

//         assert_eq!(expected_serialized, actual_serialized);

//         teardown();
//     }
// }
