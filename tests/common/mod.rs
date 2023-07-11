use std::fs;

//
// For more information about this file's naming convention, see
// https://doc.rust-lang.org/book/ch11-03-test-organization.html
//
#[allow(dead_code)]
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

// In case we want our tests to call `update`
#[allow(dead_code)]
pub fn set_up_fixtures() {
    let contains_stale_violations_bar_todo = String::from("\
# This file contains a list of dependencies that are not part of the long term plan for the
# 'packs/foo' package.
# We should generally work to reduce this list over time.
#
# You can regenerate this file using the following command:
#
# bin/packwerk update-todo
---
packs/foo:
  \"::Foo\":
    violations:
    - dependency
    - privacy
    files:
    - packs/bar/app/services/bar.rb

");

    // Rewrite tests/fixtures/contains_stale_violations/packs/bar/package_todo.yml with the above contents,
    // whether it is present or not:
    fs::write(
        "tests/fixtures/contains_stale_violations/packs/bar/package_todo.yml",
        contains_stale_violations_bar_todo,
    )
    .unwrap();

    let contains_stale_violations_foo_todo = String::from("\
# This file contains a list of dependencies that are not part of the long term plan for the
# 'packs/foo' package.
# We should generally work to reduce this list over time.
#
# You can regenerate this file using the following command:
#
# bin/packwerk update-todo
---
packs/bar:
  \"::Bar\":
    violations:
    - dependency
    - privacy
    files:
    - packs/foo/app/services/foo.rb
");

    // Rewrite tests/fixtures/contains_stale_violations/packs/bar/package_todo.yml with the above contents,
    // whether it is present or not:
    fs::write(
        "tests/fixtures/contains_stale_violations/packs/foo/package_todo.yml",
        contains_stale_violations_foo_todo,
    )
    .unwrap();
}
