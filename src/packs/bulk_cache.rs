use crate::packs::parser::get_unresolved_references;
use crate::packs::parser::UnresolvedReference;
use crate::packs::Configuration;
use crate::packs::Range;
use crate::packs::SourceLocation;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::path::PathBuf;
use tracing::debug;

use super::parser::Cache;

pub struct BulkCache {
    pub cache_dir: PathBuf,
    cache_contents: CacheEntry,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CacheEntry {
    pub per_file_cache_entry: HashMap<PathBuf, PerFileCacheEntry>,
}

pub struct PerFileCacheEntry {
    pub file_contents_digest: String,
    pub unresolved_references: Vec<UnresolvedReference>,
}

impl Cache for BulkCache {
    fn get_unresolved_references_with_cache(
        &self,
        absolute_root: &PathBuf,
        path: &PathBuf,
    ) -> Vec<UnresolvedReference> {
        let per_file_cache_entry = self
            .cache_contents
            .per_file_cache_entry
            .get(path)
            .unwrap_or_else(|| 1);
        cachable_file
            .cache_entry_if_valid()
            .map(|entry| entry.get_unresolved_references())
            .or_else(|| {
                let uncached_references = get_unresolved_references(path);
                let cloned_references = uncached_references.clone();
                write_cache(&cachable_file, cloned_references);

                Some(uncached_references)
            })
            .unwrap()
    }

    fn setup() -> Self {
        todo!()
    }

    fn teardown(&self) -> std::thread::JoinHandle<()> {
        todo!()
    }
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
                "3037a89e7de80e7a0e9543cc1ca790f9",
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
                    constant_name: String::from("::Bar"),
                    namespace_path: vec![String::from("Foo")],
                    relative_path: String::from(
                        "packs/foo/app/services/foo.rb",
                    ),
                    source_location: SourceLocation { line: 3, column: 4 },
                },
                ReferenceEntry {
                    constant_name: String::from("Baz"),
                    namespace_path: vec![String::from("Foo")],
                    relative_path: String::from(
                        "packs/foo/app/services/foo.rb",
                    ),
                    source_location: SourceLocation { line: 7, column: 4 },
                },
            ],
        };

        let absolute_root = &PathBuf::from("tests/fixtures/simple_app");
        let file_to_cache = String::from("packs/foo/app/services/foo.rb");
        let config = configuration::get(&absolute_root);

        let absolute_filepath = &PathBuf::from(
            "tests/fixtures/simple_app/packs/foo/app/services/foo.rb",
        );

        let cachable_file = CachableFile::from(
            absolute_root,
            &config.cache_directory,
            absolute_filepath,
        );

        write_cache_for_files(vec![file_to_cache.clone()], config);

        let cache_file = cachable_file.cache_file_path;
        let actual = read_json_file(&cache_file).unwrap();
        assert_eq!(actual, expected);
    }
}
