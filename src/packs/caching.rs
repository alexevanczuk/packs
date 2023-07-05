use std::path::{Path, PathBuf};

use super::{file_utils::file_content_digest, ProcessedFile};

#[derive(Debug)]
pub struct CacheMiss {
    pub relative_path: PathBuf,
    pub file_contents_digest: String,
    pub cache_file_path: PathBuf,
}

impl CacheMiss {
    // Pass in Configuration and get cache_dir from that
    pub fn new(
        absolute_root: &Path,
        cache_directory: &Path,
        filepath: &Path,
    ) -> CacheMiss {
        let relative_path: PathBuf =
            filepath.strip_prefix(absolute_root).unwrap().to_path_buf();

        let file_digest = md5::compute(relative_path.to_str().unwrap());
        let file_digest_str = format!("{:x}", file_digest);
        let cache_file_path = cache_directory.join(file_digest_str);

        let file_contents_digest = file_content_digest(filepath);

        CacheMiss {
            relative_path,
            file_contents_digest,
            cache_file_path,
        }
    }

    pub fn relative_path_string(&self) -> &str {
        self.relative_path.to_str().unwrap()
    }
}

pub trait Cache {
    // fn process_file(
    //     &self,
    //     absolute_root: &Path,
    //     path: &Path,
    //     experimental_parser: bool,
    // ) -> ProcessedFile;

    fn get(&self, absolute_root: &Path, path: &Path) -> Option<ProcessedFile>;

    fn write(&self, cache_miss: &CacheMiss, processed_file: &ProcessedFile);
}
