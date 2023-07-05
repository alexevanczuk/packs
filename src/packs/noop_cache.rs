use std::path::Path;

use super::{
    caching::{CacheResult, EmptyCacheEntry},
    ProcessedFile,
};
use crate::packs::caching::Cache;

pub struct NoopCache {}

impl Cache for NoopCache {
    fn get(&self, _absolute_root: &Path, _path: &Path) -> CacheResult {
        // Return nothing!
        CacheResult::Miss(EmptyCacheEntry::default())
    }

    fn write(
        &self,
        _empty_cache_entry: &super::caching::EmptyCacheEntry,
        _processed_file: &ProcessedFile,
    ) {
        // Do nothing!
    }
}
