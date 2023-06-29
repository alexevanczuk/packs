pub(crate) mod ruby;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use rayon::prelude::{IntoParallelIterator, ParallelIterator};
pub(crate) use ruby::packwerk::extractor::extract_from_path as extract_from_ruby_path;
pub(crate) mod erb;
pub(crate) use erb::packwerk::extractor::extract_from_path as extract_from_erb_path;

use super::ProcessedFile;

#[derive(PartialEq, Debug)]
pub enum SupportedFileType {
    Ruby,
    Erb,
}

pub fn process_file(path: &Path) -> ProcessedFile {
    let file_type_option = get_file_type(path);
    if let Some(file_type) = file_type_option {
        match file_type {
            SupportedFileType::Ruby => extract_from_ruby_path(path),
            SupportedFileType::Erb => extract_from_erb_path(path),
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

fn get_file_type(path: &Path) -> Option<SupportedFileType> {
    let ruby_special_files = vec!["Gemfile", "Rakefile"];
    let ruby_extensions = vec!["rb", "rake", "builder", "gemspec", "ru"];

    let extension = path.extension();
    // Eventually, we can have packs::parser::ruby, packs::parser::erb, etc.
    // These would implement a packs::parser::interface::Parser trait and can
    // hold the logic for determining if a parser can parse a file.

    let is_ruby_file = ruby_extensions
        .into_iter()
        .any(|ext| extension.map_or(false, |e| e == ext))
        || ruby_special_files.iter().any(|file| path.ends_with(file));

    let is_erb_file = path.extension().map_or(false, |ext| ext == "erb");

    if is_ruby_file {
        Some(SupportedFileType::Ruby)
    } else if is_erb_file {
        Some(SupportedFileType::Erb)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
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
