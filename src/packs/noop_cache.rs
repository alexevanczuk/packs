use std::path::Path;

use super::ProcessedFile;
use crate::packs::caching::Cache;

pub struct NoopCache {}

impl Cache for NoopCache {
    fn get(
        &self,
        _absolute_root: &Path,
        _path: &Path,
    ) -> Option<ProcessedFile> {
        // Return nothing!
        None
    }

    fn write(
        &self,
        _cache_miss: &super::caching::CacheMiss,
        _processed_file: &ProcessedFile,
    ) {
        // Do nothing!
    }
}
