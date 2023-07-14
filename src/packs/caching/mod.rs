use std::path::{Path, PathBuf};

use super::{file_utils::file_content_digest, ProcessedFile};
pub(crate) mod cache;
pub(crate) mod noop_cache;
pub(crate) mod per_file_cache;
pub(crate) mod bulk_cache;

pub enum CacheResult {
    Processed(ProcessedFile),
    Miss(EmptyCacheEntry),
}

#[derive(Debug, Default)]
pub struct EmptyCacheEntry {
    pub filepath: PathBuf,
    pub file_contents_digest: String,
    pub file_name_digest: String,
    pub cache_file_path: PathBuf,
}

impl EmptyCacheEntry {
    pub fn new(cache_directory: &Path, filepath: &Path) -> EmptyCacheEntry {
        let file_digest = md5::compute(filepath.to_str().unwrap());
        let file_name_digest = format!("{:x}", file_digest);
        let cache_file_path = cache_directory.join(&file_name_digest);

        let file_contents_digest = file_content_digest(filepath);

        EmptyCacheEntry {
            filepath: filepath.to_owned(),
            file_contents_digest,
            cache_file_path,
            file_name_digest,
        }
    }
}

pub fn create_cache_dir_idempotently(cache_dir: &Path) {
    std::fs::create_dir_all(cache_dir)
        .expect("Failed to create cache directory");
}
