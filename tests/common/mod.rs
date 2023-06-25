use std::fs;

//
// For more information about this file's naming convention, see
// https://doc.rust-lang.org/book/ch11-03-test-organization.html
//
pub fn teardown() {
    // Delete tmp/cache/directory across all fixture directories
    // Specifically, find directories matching the pattern:
    // tests/fixtures/*/tmp/cache/packwerk and remove that directory...

    // Here's starter code:
    glob::glob("tests/fixtures/*/tmp/cache/packwerk")
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .for_each(|cache_dir| {
            if let Err(err) = fs::remove_dir_all(&cache_dir) {
                eprintln!(
                    "Failed to remove {} during test teardown: {}",
                    &cache_dir.display(),
                    err
                );
            }
        })

    // // Remove the directory and its contents
    // if let Err(err) = fs::remove_dir_all(directory) {
    //     eprintln!(
    //         "Failed to remove tmp/cache/packwerk during test teardown: {}",
    //         err
    //     );
    // }
}
