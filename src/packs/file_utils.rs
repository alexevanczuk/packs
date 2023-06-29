use std::path::Path;

use crate::packs::parser::SupportedFileType;

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
