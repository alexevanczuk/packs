use std::path::Path;

use crate::packs::{caching::Cache, ProcessedFile};

use super::{CacheResult, EmptyCacheEntry};

pub struct NoopCache {}

impl Cache for NoopCache {
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
}
