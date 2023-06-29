pub(crate) mod ruby;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use rayon::prelude::{IntoParallelIterator, ParallelIterator};
pub(crate) use ruby::packwerk::extractor::process_from_path as process_from_ruby_path;
pub(crate) mod erb;
pub(crate) use erb::packwerk::extractor::process_from_path as process_from_erb_path;

use super::{file_utils::get_file_type, ProcessedFile};

#[derive(PartialEq, Debug)]
pub enum SupportedFileType {
    Ruby,
    Erb,
}

pub fn process_file(path: &Path) -> ProcessedFile {
    let file_type_option = get_file_type(path);
    if let Some(file_type) = file_type_option {
        match file_type {
            SupportedFileType::Ruby => process_from_ruby_path(path),
            SupportedFileType::Erb => process_from_erb_path(path),
        }
    } else {
        // Later, we can perhaps have this error, since in theory the Configuration.intersect
        // method should make sure we never get any files we can't handle.
        ProcessedFile {
            absolute_path: path.to_path_buf(),
            unresolved_references: vec![],
        }
    }
}

pub trait Cache {
    fn process_file(&self, absolute_root: &Path, path: &Path) -> ProcessedFile;
}

pub fn process_files_with_cache<T: Cache + Send + Sync>(
    absolute_root: &Path,
    paths: &HashSet<PathBuf>,
    cache: T,
) -> Vec<ProcessedFile> {
    paths
        .into_par_iter()
        .map(|absolute_path| -> ProcessedFile {
            cache.process_file(absolute_root, absolute_path)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::packs::file_utils::get_file_type;

    use super::*;

    fn assert_is_ruby(filename: &str) {
        assert_eq!(
            SupportedFileType::Ruby,
            get_file_type(Path::new(filename)).expect("Should be supported")
        )
    }

    fn assert_is_erb(filename: &str) {
        assert_eq!(
            SupportedFileType::Erb,
            get_file_type(Path::new(filename)).expect("Should be supported")
        )
    }

    #[test]
    fn identifies_ruby_files() {
        assert_is_ruby("foo.rb");
        assert_is_ruby("foo.rake");
        assert_is_ruby("Gemfile");
        assert_is_ruby("my_gem.gemspec");
    }

    #[test]
    fn identifies_erb_files() {
        assert_is_erb("foo.erb");
    }
}
