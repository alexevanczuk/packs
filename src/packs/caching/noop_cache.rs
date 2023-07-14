use std::path::Path;

use crate::packs::ProcessedFile;

use super::{cache::Cache, CacheResult, EmptyCacheEntry};

pub struct NoopCache {}

impl Cache for NoopCache {
    fn setup(&mut self, _cache_dir: &Path) {
        // Do nothing!
    }

    fn get(&self, _path: &Path) -> CacheResult {
        // Return nothing!
        CacheResult::Miss(EmptyCacheEntry::default())
    }

    fn write(
        &self,
        _empty_cache_entry: &EmptyCacheEntry,
        _processed_file: &ProcessedFile,
    ) {
        // Do nothing!
    }

    fn write_all(&self, _processed_files: &[ProcessedFile]) {
        // Do nothing!
    }
}
