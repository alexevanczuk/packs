use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use super::{
    parsing::{process_file, Cache},
    ProcessedFile,
};

pub struct NoopCache {}

impl Cache for NoopCache {
    fn process_files_with_cache(
        &self,
        _absolute_root: &Path,
        paths: &HashSet<PathBuf>,
        experimental_parser: bool,
    ) -> Vec<ProcessedFile> {
        paths
            .par_iter()
            .map(|absolute_path| -> ProcessedFile {
                process_file(absolute_path, experimental_parser)
            })
            .collect()
    }
}
