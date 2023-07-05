use std::path::Path;

use super::{parsing::process_file, ProcessedFile};
use crate::packs::caching::Cache;

pub struct NoopCache {}

impl Cache for NoopCache {
    fn process_file(
        &self,
        _absolute_root: &Path,
        path: &Path,
        experimental_parser: bool,
    ) -> ProcessedFile {
        process_file(path, experimental_parser)
    }
}
