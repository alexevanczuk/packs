use md5;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

pub(crate) fn file_content_digest(file: &PathBuf) -> String {
    let mut file_content = Vec::new();

    // Read the file content
    let mut file_handle = fs::File::open(file).unwrap_or_else(|_| panic!("Failed to open file {:?}", file));
    file_handle.read_to_end(&mut file_content).expect("Failed to read file");

    // Compute the MD5 digest
    let digest = md5::compute(&file_content);

    // Convert the digest to a hexadecimal string
    let hex_digest = format!("{:x}", digest);

    hex_digest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_content_digest() {
        let file_path = "tests/fixtures/simple_app/packs/bar/app/services/bar.rb";
        let expected_digest = "f2af2fc657b71331ff3a8c39b48365eb";

        let digest = file_content_digest(&PathBuf::from(file_path));

        assert_eq!(digest, expected_digest);
    }
}
