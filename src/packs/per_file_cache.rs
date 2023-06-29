use crate::packs::parser::process_file;
use crate::packs::SourceLocation;
use serde::{Deserialize, Serialize};

use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::path::PathBuf;

use super::parser::Cache;
use super::ProcessedFile;

pub struct PerFileCache {
    pub cache_dir: PathBuf,
}

impl Cache for PerFileCache {
    fn process_file(&self, absolute_root: &Path, path: &Path) -> ProcessedFile {
        let cachable_file =
            CachableFile::from(absolute_root, &self.cache_dir, path);

        cachable_file
            .cache_entry_if_valid()
            .map(|entry| entry.processed_file.clone())
            .or_else(|| {
                let processed_file = process_file(path);
                write_cache(&cachable_file, processed_file.clone());

                Some(processed_file)
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
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    pub file_contents_digest: String,
    pub processed_file: ProcessedFile,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceEntry {
    constant_name: String,
    namespace_path: Vec<String>,
    relative_path: String,
    source_location: SourceLocation,
}

pub fn read_json_file(
    path: &PathBuf,
) -> Result<CacheEntry, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}

pub(crate) fn file_content_digest(file: &Path) -> String {
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

fn processed_file_to_cache_entry(
    processed_file: ProcessedFile,
    cachable_file: &CachableFile,
) -> CacheEntry {
    let file_contents_digest = cachable_file.file_contents_digest.to_owned();

    CacheEntry {
        file_contents_digest,
        processed_file,
    }
}

#[derive(Debug)]
pub struct CachableFile {
    file_contents_digest: String,
    cache_file_path: PathBuf,
    cache_entry: Option<CacheEntry>,
}

impl CachableFile {
    // Pass in Configuration and get cache_dir from that
    pub fn from(
        absolute_root: &Path,
        cache_directory: &Path,
        filepath: &Path,
    ) -> CachableFile {
        let relative_path: PathBuf =
            filepath.strip_prefix(absolute_root).unwrap().to_path_buf();

        let file_digest = md5::compute(relative_path.to_str().unwrap());
        let file_digest_str = env::var("CACHE_VERIFICATION")
            .map(|_| format!("{:x}-experimental", file_digest))
            .unwrap_or_else(|_| format!("{:x}", file_digest));

        let cache_file_path = cache_directory.join(file_digest_str);

        let file_contents_digest = file_content_digest(filepath);

        let cache_entry: Option<CacheEntry> = if cache_file_path.exists() {
            Some(read_json_file(&cache_file_path).unwrap_or_else(|_| {
                panic!("Failed to read cache file {:?}", cache_file_path)
            }))
        } else {
            None
        };

        CachableFile {
            file_contents_digest,
            cache_file_path,
            cache_entry,
        }
    }

    pub fn cache_entry_if_valid(&self) -> Option<&CacheEntry> {
        if self.cache_is_valid() {
            self.cache_entry.as_ref()
        } else {
            None
        }
    }

    pub fn cache_is_valid(&self) -> bool {
        if let Some(cache_entry) = &self.cache_entry {
            cache_entry.file_contents_digest == self.file_contents_digest
        } else {
            false
        }
    }
}

fn write_cache(cachable_file: &CachableFile, processed_file: ProcessedFile) {
    let cache_entry =
        processed_file_to_cache_entry(processed_file, cachable_file);

    let cache_data = serde_json::to_string(&cache_entry)
        .expect("Failed to serialize references");
    let mut file =
        File::create(&cachable_file.cache_file_path).unwrap_or_else(|e| {
            panic!(
                "Failed to create cache file {:?}: {}",
                cachable_file.cache_file_path, e
            )
        });

    file.write_all(cache_data.as_bytes())
        .expect("Failed to write cache file");
}

pub fn create_cache_dir_idempotently(cache_dir: &PathBuf) {
    std::fs::create_dir_all(cache_dir)
        .expect("Failed to create cache directory");
}

#[cfg(test)]
mod tests {
    use crate::packs::{self, configuration};

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
        let expected_digest = "f2af2fc657b71331ff3a8c39b48365eb";

        let digest = file_content_digest(&PathBuf::from(file_path));

        assert_eq!(digest, expected_digest);

        teardown();
    }
}
