use std::path::Path;

use super::{
    parsing::{process_file, Cache},
    ProcessedFile,
};

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
