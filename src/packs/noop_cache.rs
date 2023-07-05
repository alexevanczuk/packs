use std::path::Path;

use super::{
    caching::{CacheMiss, CacheResult},
    ProcessedFile,
};
use crate::packs::caching::Cache;

pub struct NoopCache {}

impl Cache for NoopCache {
    fn get(&self, _absolute_root: &Path, _path: &Path) -> CacheResult {
        // Return nothing!
        CacheResult::Miss(CacheMiss::default())
    }

    fn write(
        &self,
        _cache_miss: &super::caching::CacheMiss,
        _processed_file: &ProcessedFile,
    ) {
        // Do nothing!
    }
}
