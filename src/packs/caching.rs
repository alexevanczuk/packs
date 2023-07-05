use std::path::Path;

use super::ProcessedFile;

pub trait Cache {
    fn process_file(
        &self,
        absolute_root: &Path,
        path: &Path,
        experimental_parser: bool,
    ) -> ProcessedFile;
}
