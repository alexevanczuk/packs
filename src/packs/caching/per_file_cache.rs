use crate::packs::ProcessedFile;
use serde::{Deserialize, Serialize};

use anyhow::Context;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use tracing::warn;

use super::cache::Cache;
use super::CacheResult;
use super::EmptyCacheEntry;

pub struct PerFileCache {
    pub cache_dir: PathBuf,
}

impl Cache for PerFileCache {
    fn get(&self, path: &Path) -> anyhow::Result<CacheResult> {
        let empty_cache_entry = EmptyCacheEntry::new(&self.cache_dir, path)
            .context(format!("Failed to create cache entry for {:?}", path))?;
        let cache_entry = CacheEntry::from_empty(&empty_cache_entry)?;
        if let Some(cache_entry) = cache_entry {
            let file_digests_match = cache_entry.file_contents_digest
                == empty_cache_entry.file_contents_digest;

            if !file_digests_match {
                Ok(CacheResult::Miss(empty_cache_entry))
            } else {
                let processed_file = cache_entry.processed_file;
                Ok(CacheResult::Processed(processed_file))
            }
        } else {
            Ok(CacheResult::Miss(empty_cache_entry))
        }
    }

    fn write(
        &self,
        empty_cache_entry: &EmptyCacheEntry,
        processed_file: &ProcessedFile,
    ) -> anyhow::Result<()> {
        let file_contents_digest =
            empty_cache_entry.file_contents_digest.to_owned();

        let cache_entry = &CacheEntry {
            file_contents_digest,
            // Ideally we could pass by reference here, but in practice this cost should be paid on few files
            // that have changed and need to be reprocessed.
            processed_file: processed_file.clone(),
        };

        let cache_data = serde_json::to_string(&cache_entry)
            .context("Failed to serialize references")?;
        let mut file = File::create(&empty_cache_entry.cache_file_path)
            .map_err(|e| {
                anyhow::Error::new(e).context(format!(
                    "Failed to create cache file {:?}",
                    empty_cache_entry.cache_file_path
                ))
            })?;

        file.write_all(cache_data.as_bytes())
            .context("Failed to write cache file")?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheEntry {
    pub file_contents_digest: String,
    pub processed_file: ProcessedFile,
}

impl CacheEntry {
    pub fn from_empty(
        empty: &EmptyCacheEntry,
    ) -> anyhow::Result<Option<CacheEntry>> {
        let cache_file_path = &empty.cache_file_path;

        if cache_file_path.exists() {
            match read_json_file(cache_file_path) {
                Ok(cache_entry) => Ok(Some(cache_entry)),
                Err(e) => {
                    warn!(
                        "Failed to read cache file {:?}: {}",
                        cache_file_path, e
                    );
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }
}

pub fn read_json_file(path: &PathBuf) -> anyhow::Result<CacheEntry> {
    let file = std::fs::File::open(path)
        .context(format!("Failed to open file {:?}", path))?;
    let reader = std::io::BufReader::new(file);
    let data = serde_json::from_reader(reader)
        .context("Failed to deserialize CacheEntry")?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::packs::{
        self, configuration,
        file_utils::file_content_digest,
        parsing::{Range, UnresolvedReference},
    };

    use super::*;

    fn teardown() {
        packs::delete_cache(
            configuration::get(&PathBuf::from("tests/fixtures/simple_app"))
                .unwrap(),
        );
    }

    #[test]
    fn test_file_content_digest() {
        let file_path =
            "tests/fixtures/simple_app/packs/bar/app/services/bar.rb";
        let expected_digest = "305bc58696c2e664057b6751064cf2e3";

        let digest = file_content_digest(&PathBuf::from(file_path));

        assert!(digest.is_ok());
        assert_eq!(digest.unwrap(), expected_digest);

        teardown();
    }

    #[test]
    fn test_compatible_with_packwerk() {
        let contents: String = String::from(
            r#"{
  "file_contents_digest":"8f9efdcf2caa22fb7b1b4a8274e68d11",
  "processed_file": {
    "absolute_path":"/tests/fixtures/simple_app/packs/foo/app/services/bar/foo.rb",
    "unresolved_references":[
      {
        "name":"Bar",
        "namespace_path":["Foo","Bar"],
        "location":{"start_row":8,"start_col":22,"end_row":8,"end_col":25}
      }],
    "definitions":[]
  }
}"#,
        );

        let expected_serialized = CacheEntry {
            file_contents_digest: "8f9efdcf2caa22fb7b1b4a8274e68d11".to_owned(),
            processed_file: ProcessedFile {
                absolute_path: PathBuf::from("/tests/fixtures/simple_app/packs/foo/app/services/bar/foo.rb"),
                unresolved_references: vec![UnresolvedReference {
                    name: "Bar".to_owned(),
                    namespace_path: vec!["Foo".to_owned(), "Bar".to_owned()],
                    location: Range {
                        start_row: 8,
                        start_col: 22,
                        end_row: 8,
                        end_col: 25,
                    }
                }],
                definitions: vec![],
            }
        };

        let actual_serialized =
            serde_json::from_str::<CacheEntry>(&contents).unwrap();

        assert_eq!(expected_serialized, actual_serialized);

        teardown();
    }

    #[test]
    fn test_corrupt_cache() -> anyhow::Result<()> {
        let sha = "e57a05216069923190a4e03d264d9677";
        let corrupt_contents: String = String::from(
            r#"{
  "file_contents_digest":"e57a05216069923190a4e03d264d9677",
  "processed_file": 
}"#,
        );

        let cache_path = PathBuf::from("tests/fixtures/simple_app/tmp/cache/");
        fs::create_dir_all(&cache_path)
            .context("unable to create cache dir")?;
        let corrupt_file_path = cache_path.join(sha);
        fs::write(corrupt_file_path, corrupt_contents)
            .context("expected to write corrupt cache file")?;

        let empty_cache_entry = EmptyCacheEntry::new(
            &cache_path,
            &PathBuf::from(
                "tests/fixtures/simple_app/packs/foo/app/services/foo/bar.rb",
            ),
        ).context("expected tests/fixtures/simple_app/packs/foo/app/services/foo/bar.rb to exist")?;

        let entry = CacheEntry::from_empty(&empty_cache_entry)?;
        assert!(entry.is_none());

        Ok(())
    }
}
