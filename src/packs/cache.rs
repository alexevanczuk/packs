use crate::packs::parser::extract_from_ruby_path;
use crate::packs::parser::UnresolvedReference;
use crate::packs::Configuration;
use crate::packs::Range;
use crate::packs::SourceLocation;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct CacheEntry {
    file_contents_digest: String,
    unresolved_references: Vec<ReferenceEntry>,
}

impl CacheEntry {
    fn get_unresolved_references(&self) -> Vec<UnresolvedReference> {
        self.unresolved_references
            .iter()
            .map(|r| -> UnresolvedReference { r.to_unresolved_reference() })
            .collect()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ReferenceEntry {
    constant_name: String,
    namespace_path: Vec<String>,
    relative_path: String,
    source_location: SourceLocation,
}

impl ReferenceEntry {
    fn to_unresolved_reference(&self) -> UnresolvedReference {
        UnresolvedReference {
            name: self.constant_name.to_owned(),
            namespace_path: self.namespace_path.to_owned(),
            location: Range {
                start_row: self.source_location.line,
                start_col: self.source_location.column,
                // The end row and end col can be improved here but we are limited
                // because the cache does not store this data.
                // Instead, we might just return a (resolved) Reference
                end_row: self.source_location.line,
                end_col: self.source_location.column + self.constant_name.len(),
            },
        }
    }
}
pub fn get_unresolved_references(
    configuration: &Configuration,
    path: &PathBuf,
) -> Vec<UnresolvedReference> {
    let current_file_contents_digest = file_content_digest(path);
    let relative_path =
        path.strip_prefix(&configuration.absolute_root).unwrap();

    let filename_digest =
        format!("{:?}", md5::compute(relative_path.to_str().unwrap()));
    let cache_dir = configuration.absolute_root.join("tmp/cache/packwerk");
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
    write_cache(&configuration.absolute_root, path, references.clone());

    references
}

// Used for tests, for now!
#[allow(dead_code)]
fn read_json_file(
    path: &PathBuf,
) -> Result<CacheEntry, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}

pub(crate) fn file_content_digest(file: &PathBuf) -> String {
    let mut file_content = Vec::new();

    // Read the file content
    let mut file_handle = fs::File::open(file)
        .unwrap_or_else(|_| panic!("Failed to open file {:?}", file));
    file_handle
        .read_to_end(&mut file_content)
        .expect("Failed to read file");

    // Compute the MD5 digest
    format!("{:x}", md5::compute(&file_content))
}

fn references_to_cache_entry(
    references: Vec<UnresolvedReference>,
    file_contents_digest: String,
    relative_path: String,
) -> CacheEntry {
    let unresolved_references: Vec<ReferenceEntry> = references
        .iter()
        .map(|r| -> ReferenceEntry {
            ReferenceEntry {
                constant_name: r.name.to_owned(),
                namespace_path: r.namespace_path.to_owned(),
                relative_path: relative_path.to_owned(),
                source_location: SourceLocation {
                    line: r.location.start_row,
                    column: r.location.start_col,
                },
            }
        })
        .collect();
    CacheEntry {
        file_contents_digest,
        unresolved_references,
    }
}

fn write_cache(
    absolute_root: &Path,
    path_to_file_being_cached: &Path,
    references: Vec<UnresolvedReference>,
) {
    let relative_path_to_file = path_to_file_being_cached
        .strip_prefix(absolute_root)
        .unwrap();
    let absolute_path = absolute_root.join(relative_path_to_file);

    let cache_dir = absolute_root.join("tmp/cache/packwerk");
    std::fs::create_dir_all(&cache_dir)
        .expect("Failed to create cache directory");

    let file_digest = md5::compute(relative_path_to_file.to_str().unwrap());
    let file_digest_str = env::var("CACHE_VERIFICATION")
        .map(|_| format!("{:x}-experimental", file_digest))
        .unwrap_or_else(|_| format!("{:x}", file_digest));

    let cache_file_path = cache_dir.join(file_digest_str);
    let cache_entry = references_to_cache_entry(
        references,
        file_content_digest(&absolute_path),
        relative_path_to_file
            .to_str()
            .expect("Could not convert cache_file_path to string")
            .to_string(),
    );

    let cache_data = serde_json::to_string(&cache_entry)
        .expect("Failed to serialize references");
    let mut file =
        File::create(cache_file_path).expect("Failed to create cache file");
    file.write_all(cache_data.as_bytes())
        .expect("Failed to write cache file");
}

fn parse_path_for_references(path: &PathBuf) -> Vec<UnresolvedReference> {
    let ruby_special_files = vec!["Gemfile", "Rakefile"];
    let ruby_extensions = vec!["rb", "rake", "builder", "gemspec", "ru"];

    // Eventually, we can have packs::parser::ruby, packs::parser::erb, etc.
    // These would implement a packs::parser::interface::Parser trait and can
    // hold the logic for determining if a parser can parse a file.
    let is_ruby_file = ruby_extensions
        .into_iter()
        .any(|ext| path.extension().unwrap() == ext)
        || ruby_special_files.iter().any(|file| path.ends_with(file));

    let is_erb_file = path.ends_with("erb");

    if is_ruby_file {
        extract_from_ruby_path(path)
    } else if is_erb_file {
        todo!();
    } else {
        // Later, we can perhaps have this error, since in theory the Configuration.intersect
        // method should make sure we never get any files we can't handle.
        vec![]
    }
}
pub(crate) fn write_cache_for_files(
    files: Vec<String>,
    configuration: Configuration,
) {
    let absolute_paths: HashSet<PathBuf> = configuration.intersect_files(files);
    let absolute_root_path = configuration.absolute_root;

    absolute_paths.par_iter().for_each(|path| {
        let references = parse_path_for_references(path);
        write_cache(&absolute_root_path, path, references);
    })
}

#[cfg(test)]
mod tests {
    use crate::packs::configuration;

    use super::*;

    #[test]
    fn test_file_content_digest() {
        let file_path =
            "tests/fixtures/simple_app/packs/bar/app/services/bar.rb";
        let expected_digest = "f2af2fc657b71331ff3a8c39b48365eb";

        let digest = file_content_digest(&PathBuf::from(file_path));

        assert_eq!(digest, expected_digest);
    }

    #[test]
    fn test_write_cache_for_files() {
        let expected = CacheEntry {
            file_contents_digest: String::from(
                // This is the MD5 digest of the contents of "packs/foo/app/services/foo.rb"
                // i.e. in ruby, it's:
                // Digest::MD5.hexdigest(File.read('tests/fixtures/simple_app/packs/foo/app/services/foo.rb'))
                "f24fb260c246613675488000115037c0",
            ),
            unresolved_references: vec![
                ReferenceEntry {
                    constant_name: String::from("::Foo"),
                    namespace_path: vec![String::from("Foo")],
                    relative_path: String::from(
                        "packs/foo/app/services/foo.rb",
                    ),
                    source_location: SourceLocation { line: 1, column: 7 },
                },
                ReferenceEntry {
                    constant_name: String::from("Bar"),
                    namespace_path: vec![String::from("Foo")],
                    relative_path: String::from(
                        "packs/foo/app/services/foo.rb",
                    ),
                    source_location: SourceLocation { line: 3, column: 4 },
                },
                ReferenceEntry {
                    constant_name: String::from("Baz"),
                    namespace_path: vec![String::from("Foo")],
                    relative_path: String::from(
                        "packs/foo/app/services/foo.rb",
                    ),
                    source_location: SourceLocation { line: 7, column: 4 },
                },
            ],
        };

        write_cache_for_files(
            vec![String::from("packs/foo/app/services/foo.rb")],
            configuration::get(&PathBuf::from("tests/fixtures/simple_app")),
        );

        let cache_file = PathBuf::from("tests/fixtures/simple_app/tmp/cache/packwerk/061bf98e1706eac5af59c4b1a770fc7e");
        let actual = read_json_file(&cache_file).unwrap();
        assert_eq!(actual, expected);
    }
}
