use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use regex::Regex;

#[derive(PartialEq, Debug)]
pub enum SupportedFileType {
    Ruby,
    Erb,
}

pub fn get_file_type(path: &Path) -> Option<SupportedFileType> {
    let ruby_special_files = vec!["Gemfile", "Rakefile"];
    let ruby_extensions = vec!["rb", "rake", "builder", "gemspec", "ru"];

    let extension = path.extension();
    // Eventually, we can have packs::parsing::ruby, packs::parsing::erb, etc.
    // These would implement a packs::parsing::interface::Parser trait and can
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

pub fn build_glob_set(globs: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();

    for glob in globs {
        let compiled_glob = GlobBuilder::new(glob)
            .literal_separator(true)
            .build()
            .unwrap();

        builder.add(compiled_glob);
    }

    builder.build().unwrap()
}

pub fn process_glob_pattern(pattern: &str, paths: &mut Vec<PathBuf>) {
    for path in glob::glob(pattern)
        .expect("Failed to read glob pattern")
        .flatten()
    {
        paths.push(path);
    }
}

pub fn user_inputted_paths_to_absolute_filepaths(
    absolute_root: &Path,
    input_paths: Vec<String>,
) -> HashSet<PathBuf> {
    input_paths
        .iter()
        .map(PathBuf::from)
        .flat_map(|p| {
            if p.is_absolute() {
                vec![p]
            } else {
                let absolute_path = absolute_root.join(&p);
                if absolute_path.is_dir() {
                    glob::glob(absolute_path.join("**/*.*").to_str().unwrap())
                        .expect("Failed to read glob pattern")
                        .filter_map(Result::ok)
                        .collect::<Vec<_>>()
                } else {
                    vec![absolute_path]
                }
            }
        })
        .collect::<HashSet<_>>()
}

pub(crate) fn convert_erb_to_mangled_ruby(contents: String) -> String {
    let regex_pattern = r#"(?s)<%=?-?\s*(.*?)\s*-?%>"#;
    let regex = Regex::new(regex_pattern).unwrap();

    let extracted_contents: Vec<&str> = regex
        .captures_iter(&contents)
        .map(|capture| capture.get(1).unwrap().as_str())
        .collect();

    extracted_contents.join("\n")
}
