use crate::packs::parser::get_unresolved_references;
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
use std::path::Path;
use std::path::PathBuf;
use tracing::debug;

use super::parser::Cache;

pub struct PerFileCache {
    pub cache_dir: PathBuf,
}

impl Cache for PerFileCache {
    fn get_unresolved_references_with_cache(
        &self,
        absolute_root: &Path,
        path: &Path,
    ) -> Vec<UnresolvedReference> {
        let cachable_file =
            CachableFile::from(absolute_root, &self.cache_dir, path);
        cachable_file
            .cache_entry_if_valid()
            .map(|entry| entry.get_unresolved_references())
            .or_else(|| {
                let uncached_references = get_unresolved_references(path);
                let cloned_references = uncached_references.clone();
                write_cache(&cachable_file, cloned_references);

                Some(uncached_references)
            })
            .unwrap()
    }

    fn setup() -> Self {
        todo!()
    }

    fn teardown(&self) -> std::thread::JoinHandle<()> {
        todo!()
    }
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CacheEntry {
    pub file_contents_digest: String,
    pub unresolved_references: Vec<ReferenceEntry>,
}

impl CacheEntry {
    pub fn get_unresolved_references(&self) -> Vec<UnresolvedReference> {
        self.unresolved_references
            .iter()
            .map(|r| -> UnresolvedReference { r.to_unresolved_reference() })
            .collect()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceEntry {
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

pub fn read_json_file(
    path: &PathBuf,
) -> Result<CacheEntry, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}

pub(crate) fn file_content_digest(file: &Path) -> String {
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
    cachable_file: &CachableFile,
) -> CacheEntry {
    let unresolved_references: Vec<ReferenceEntry> = references
        .iter()
        .map(|r| -> ReferenceEntry {
            ReferenceEntry {
                constant_name: r.name.to_owned(),
                namespace_path: r.namespace_path.to_owned(),
                relative_path: cachable_file.relative_path_string().to_owned(),
                source_location: SourceLocation {
                    line: r.location.start_row,
                    column: r.location.start_col,
                },
            }
        })
        .collect();

    let file_contents_digest = cachable_file.file_contents_digest.to_owned();

    CacheEntry {
        file_contents_digest,
        unresolved_references,
    }
}

#[derive(Debug)]
pub struct CachableFile {
    relative_path: PathBuf,
    file_contents_digest: String,
    cache_file_path: PathBuf,
    cache_entry: Option<CacheEntry>,
}

impl CachableFile {
    // Pass in Configuration and get cache_dir from that
    pub fn from(
        absolute_root: &Path,
        cache_directory: &Path,
        filepath: &Path,
    ) -> CachableFile {
        let relative_path: PathBuf =
            filepath.strip_prefix(absolute_root).unwrap().to_path_buf();

        let file_digest = md5::compute(relative_path.to_str().unwrap());
        let file_digest_str = env::var("CACHE_VERIFICATION")
            .map(|_| format!("{:x}-experimental", file_digest))
            .unwrap_or_else(|_| format!("{:x}", file_digest));

        let cache_file_path = cache_directory.join(file_digest_str);

        let file_contents_digest = file_content_digest(filepath);

        let cache_entry: Option<CacheEntry> = if cache_file_path.exists() {
            Some(read_json_file(&cache_file_path).unwrap_or_else(|_| {
                panic!("Failed to read cache file {:?}", cache_file_path)
            }))
        } else {
            None
        };

        CachableFile {
            relative_path,
            file_contents_digest,
            cache_file_path,
            cache_entry,
        }
    }

    fn relative_path_string(&self) -> &str {
        self.relative_path.to_str().unwrap()
    }

    pub fn cache_entry_if_valid(&self) -> Option<&CacheEntry> {
        if self.cache_is_valid() {
            self.cache_entry.as_ref()
        } else {
            None
        }
    }

    pub fn cache_is_valid(&self) -> bool {
        if let Some(cache_entry) = &self.cache_entry {
            cache_entry.file_contents_digest == self.file_contents_digest
        } else {
            false
        }
    }
}

fn write_cache(
    cachable_file: &CachableFile,
    references: Vec<UnresolvedReference>,
) {
    let cache_entry = references_to_cache_entry(references, cachable_file);

    let cache_data = serde_json::to_string(&cache_entry)
        .expect("Failed to serialize references");
    let mut file =
        File::create(&cachable_file.cache_file_path).unwrap_or_else(|e| {
            panic!(
                "Failed to create cache file {:?}: {}",
                cachable_file.cache_file_path, e
            )
        });

    file.write_all(cache_data.as_bytes())
        .expect("Failed to write cache file");
}

pub fn create_cache_dir_idempotently(cache_dir: &PathBuf) {
    std::fs::create_dir_all(cache_dir)
        .expect("Failed to create cache directory");
}

pub(crate) fn write_cache_for_files(
    files: Vec<String>,
    configuration: Configuration,
) {
    create_cache_dir_idempotently(&configuration.cache_directory);

    let absolute_paths: HashSet<PathBuf> = configuration.intersect_files(files);
    let file_count = absolute_paths.len();
    debug!("Writing cache for {} files", file_count);

    absolute_paths.par_iter().for_each(|path| {
        let cachable_file = CachableFile::from(
            &configuration.absolute_root,
            &configuration.cache_directory,
            path,
        );
        if !cachable_file.cache_is_valid() {
            let references = get_unresolved_references(path);
            write_cache(&cachable_file, references)
        }
    });
    debug!("Finished writing cache for {} files", file_count);
}

#[cfg(test)]
mod tests {
    use crate::packs::{self, configuration};

    use super::*;

    fn teardown() {
        packs::delete_cache(configuration::get(&PathBuf::from(
            "tests/fixtures/simple_app",
        )));
    }

    #[test]
    fn test_file_content_digest() {
        let file_path =
            "tests/fixtures/simple_app/packs/bar/app/services/bar.rb";
        let expected_digest = "f2af2fc657b71331ff3a8c39b48365eb";

        let digest = file_content_digest(&PathBuf::from(file_path));

        assert_eq!(digest, expected_digest);

        teardown();
    }

    #[test]
    fn test_write_cache_for_files() {
        let absolute_root = &PathBuf::from("tests/fixtures/simple_app");

        // Delete the existing cache directory
        // TODO: This is a bit of a hack, we should probably use a temporary directory
        // instead of the real one
        let cache_dir = absolute_root.join("tmp");
        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir).unwrap();
        }

        let expected = CacheEntry {
            file_contents_digest: String::from(
                // This is the MD5 digest of the contents of "packs/foo/app/services/foo.rb"
                // i.e. in ruby, it's:
                // Digest::MD5.hexdigest(File.read('tests/fixtures/simple_app/packs/foo/app/services/foo.rb'))
                "3037a89e7de80e7a0e9543cc1ca790f9",
            ),
            unresolved_references: vec![
                ReferenceEntry {
                    constant_name: String::from("::Foo"),
                    namespace_path: vec![],
                    relative_path: String::from(
                        "packs/foo/app/services/foo.rb",
                    ),
                    source_location: SourceLocation { line: 1, column: 7 },
                },
                ReferenceEntry {
                    constant_name: String::from("::Bar"),
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

        let file_to_cache = String::from("packs/foo/app/services/foo.rb");
        let config = configuration::get(absolute_root);

        let absolute_filepath = &PathBuf::from(
            "tests/fixtures/simple_app/packs/foo/app/services/foo.rb",
        );

        let cachable_file = CachableFile::from(
            absolute_root,
            &config.cache_directory,
            absolute_filepath,
        );

        write_cache_for_files(vec![file_to_cache], config);

        let cache_file = cachable_file.cache_file_path;
        let actual = read_json_file(&cache_file).unwrap();
        assert_eq!(expected, actual);

        teardown();
    }
}
