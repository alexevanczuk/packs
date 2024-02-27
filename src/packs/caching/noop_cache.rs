use std::path::Path;

use crate::packs::ProcessedFile;

use super::{cache::Cache, CacheResult, EmptyCacheEntry};

pub struct NoopCache {}

impl Cache for NoopCache {
    fn get(&self, _path: &Path) -> anyhow::Result<CacheResult> {
        // Return nothing!
        Ok(CacheResult::Miss(EmptyCacheEntry::default()))
    }

    fn write(
        &self,
        _empty_cache_entry: &EmptyCacheEntry,
        _processed_file: &ProcessedFile,
    ) {
        // Do nothing!
    }
}
