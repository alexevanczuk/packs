use std::path::{Path, PathBuf};

use super::{file_utils::file_content_digest, ProcessedFile};

pub enum CacheResult {
    Processed(ProcessedFile),
    Miss(EmptyCacheEntry),
}

pub(crate) mod noop_cache;
pub(crate) mod per_file_cache;

#[derive(Debug, Default)]
pub struct EmptyCacheEntry {
    pub relative_path: PathBuf,
    pub file_contents_digest: String,
    pub file_name_digest: String,
    pub cache_file_path: PathBuf,
}

impl EmptyCacheEntry {
    pub fn new(
        absolute_root: &Path,
        cache_directory: &Path,
        filepath: &Path,
    ) -> EmptyCacheEntry {
        let relative_path: PathBuf =
            filepath.strip_prefix(absolute_root).unwrap().to_path_buf();

        let file_digest = md5::compute(relative_path.to_str().unwrap());
        let file_name_digest = format!("{:x}", file_digest);
        let cache_file_path = cache_directory.join(&file_name_digest);

        let file_contents_digest = file_content_digest(filepath);

        EmptyCacheEntry {
            relative_path,
            file_contents_digest,
            cache_file_path,
            file_name_digest,
        }
    }

    pub fn relative_path_string(&self) -> &str {
        self.relative_path.to_str().unwrap()
    }
}

pub trait Cache {
    fn get(&self, absolute_root: &Path, path: &Path) -> CacheResult;

    fn write(
        &self,
        empty_cache_entry: &EmptyCacheEntry,
        processed_file: &ProcessedFile,
    );
}

pub fn create_cache_dir_idempotently(cache_dir: &Path) {
    std::fs::create_dir_all(cache_dir)
        .expect("Failed to create cache directory");
}
