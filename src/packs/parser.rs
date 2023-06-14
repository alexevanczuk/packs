pub(crate) mod ruby;
use std::path::{Path, PathBuf};

pub(crate) use ruby::packwerk::extractor::extract_from_path as extract_from_ruby_path;
pub(crate) mod erb;
pub(crate) use erb::packwerk::extractor::extract_from_path as extract_from_erb_path;

// TODO: Move this somewhere else
pub(crate) use ruby::packwerk::extractor::UnresolvedReference;

use crate::packs::cache::{
    file_content_digest, read_json_file, write_cache, CachableFile,
};

#[derive(PartialEq, Debug)]
pub enum SupportedFileType {
    Ruby,
    Erb,
}

pub fn parse_path_for_references(path: &PathBuf) -> Vec<UnresolvedReference> {
    let file_type_option = get_file_type(path);
    if let Some(file_type) = file_type_option {
        match file_type {
            SupportedFileType::Ruby => extract_from_ruby_path(path),
            SupportedFileType::Erb => extract_from_erb_path(path),
        }
    } else {
        // Later, we can perhaps have this error, since in theory the Configuration.intersect
        // method should make sure we never get any files we can't handle.
        vec![]
    }
}

// TODO: parse_path_for_references should accept a cache trait type (default no-op) and process
// cache related activities within the implementation of the trait
pub fn get_unresolved_references(
    absolute_root: &PathBuf,
    cache_dir: &Path,
    path: &PathBuf,
) -> Vec<UnresolvedReference> {
    let current_file_contents_digest = file_content_digest(path);
    let relative_path = path.strip_prefix(absolute_root).unwrap();

    let filename_digest =
        format!("{:?}", md5::compute(relative_path.to_str().unwrap()));
    let cache_path = cache_dir.join(filename_digest);

    if cache_path.exists() {
        let cache = read_json_file(&cache_path).unwrap_or_else(|_| {
            panic!("Failed to read cache file {:?}", cache_path)
        });
        if cache.file_contents_digest == current_file_contents_digest {
            return cache.get_unresolved_references();
        }
    }

    let references = parse_path_for_references(path);
    // TODO: This work can be done in a new thread;
    let cachable_file = CachableFile::from(absolute_root, cache_dir, path);
    write_cache(&cachable_file, references.clone());

    references
}

pub fn get_file_type(path: &Path) -> Option<SupportedFileType> {
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
