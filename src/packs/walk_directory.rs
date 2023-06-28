use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use jwalk::WalkDirGeneric;
use std::{collections::HashSet, path::PathBuf, sync::Arc};

use super::configuration::RawConfiguration;
use crate::packs::Pack;

fn build_glob_set(globs: &[String]) -> GlobSet {
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

pub struct WalkDirectoryResult {
    pub included_paths: HashSet<PathBuf>,
    pub included_packs: HashSet<Pack>,
}

// We use jwalk to walk directories in parallel and compare them to the `include` and `exclude` patterns
// specified in the `RawConfiguration`
// https://docs.rs/jwalk/0.8.1/jwalk/struct.WalkDirGeneric.html#method.process_read_dir
// We only walk the directory once and pull all of the information we need from it,
// which is faster than walking the directory multiple times.
// Likely, we can organize this better by moving each piece of logic into its own function so this function
// allows for a sort of "visitor pattern" for different things that need to walk the directory.
pub fn walk_directory(
    absolute_root: PathBuf,
    raw: &RawConfiguration,
) -> WalkDirectoryResult {
    let mut included_paths: HashSet<PathBuf> = HashSet::new();
    let mut included_packs: HashSet<Pack> = HashSet::new();
    // Create this vector outside of the closure to avoid reallocating it
    let default_excluded_dirs = vec![
        "node_modules/**/*",
        "vendor/**/*",
        "tmp/**/*",
        ".git/**/*",
        "public/**/*",
        "bin/**/*",
        "log/**/*",
        "frontend/**/**",
        "sorbet/**/*",
    ];
    let mut all_excluded_dirs: Vec<String> = Vec::new();
    all_excluded_dirs
        .extend(default_excluded_dirs.iter().map(|s| s.to_string()));

    let excluded_globs = &raw.exclude;
    all_excluded_dirs.extend(excluded_globs.to_owned());

    let all_excluded_dirs_set = build_glob_set(&all_excluded_dirs);
    let excluded_dirs_ref = Arc::new(all_excluded_dirs_set);

    let absolute_root_ref = Arc::new(absolute_root.clone());

    let includes_set = build_glob_set(&raw.include);
    let excludes_set = build_glob_set(&raw.exclude);
    let package_paths_set = build_glob_set(&raw.package_paths);

    // TODO: Pull directory walker into separate module. Allow it to be called with implementations of a trait
    // so separate concerns can each be in their own place.
    //
    // WalkDirGeneric allows you to customize the directory walk, such as skipping directories,
    // which we do as a performance optimization.
    //
    // Specifically – if an exclude glob matches an entire directory, we don't need to continue to
    // explore it. For example, instead of asking every file in `vendor/bundle/**/` if it should be excluded,
    // we'll save a lot of time by just skipping the entire directory.
    //
    // For more information, check out the docs: https://docs.rs/jwalk/0.8.1/jwalk/#extended-example
    let walk_dir = WalkDirGeneric::<(usize, bool)>::new(&absolute_root)
        .process_read_dir(move |_depth, _path, _read_dir_state, children| {
            // We need to let the compiler know that we are using a reference and not the value itself.
            // We need to then clone the Arc to get a new reference, which is a new pointer to the value/data
            // (with an increase to the reference count).
            let cloned_excluded_dirs = excluded_dirs_ref.clone();
            let cloned_absolute_root = absolute_root_ref.clone();

            children.iter_mut().for_each(|dir_entry_result| {
                if let Ok(dir_entry) = dir_entry_result {
                    let absolute_dirname = dir_entry.path();
                    let relative_path = absolute_dirname
                        .strip_prefix(cloned_absolute_root.as_ref())
                        .unwrap()
                        .to_owned();

                    if cloned_excluded_dirs.as_ref().is_match(relative_path) {
                        dir_entry.read_children_path = None;
                    }
                }
            });
        });

    for entry in walk_dir {
        // I was using this to explore what directories were being walked to potentially
        // find performance improvements.
        // Write the entry out to a log file:
        // use std::io::Write;
        // let mut file = std::fs::OpenOptions::new()
        //     .create(true)
        //     .append(true)
        //     .open("tmp/pks_log.txt")
        //     .unwrap();
        // writeln!(file, "{:?}", entry).unwrap();
        let unwrapped_entry = entry.unwrap();

        // Note that we could also get the dir from absolute_path.is_dir()
        // However, this data appears to be cached on the FileType struct, so we'll use that instead,
        // which is much faster!
        if unwrapped_entry.file_type.is_dir() {
            continue;
        }

        let absolute_path = unwrapped_entry.path();

        let relative_path = absolute_path
            .strip_prefix(&absolute_root)
            .unwrap()
            .to_owned();

        // This could be one line, but I'm keeping it separate for debugging purposes
        if includes_set.is_match(&relative_path) {
            if !excludes_set.is_match(&relative_path) {
                included_paths.insert(absolute_path.clone());
            } else {
                // println!("file excluded: {}", relative_path.display())
            }
        } else {
            // println!(
            //     "file not included: {:?}, {:?}",
            //     relative_path.display(),
            //     &raw.include
            // )
        }

        let file_name =
            relative_path.file_name().expect("expected a file_name");

        if file_name.eq_ignore_ascii_case("package.yml")
            && (package_paths_set.is_match(relative_path.parent().unwrap())
                || absolute_path.parent().unwrap() == absolute_root)
        {
            let pack = Pack::from_path(&absolute_path, &relative_path);
            included_packs.insert(pack);
        }
    }

    WalkDirectoryResult {
        included_paths,
        included_packs,
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, path::PathBuf};

    use crate::packs::{
        configuration::RawConfiguration,
        walk_directory::{walk_directory, WalkDirectoryResult},
    };

    #[test]
    fn test_walk_directory() -> Result<(), Box<dyn Error>> {
        let absolute_path = PathBuf::from("tests/fixtures/simple_app")
            .canonicalize()
            .expect("Could not canonicalize path");

        let raw_config = RawConfiguration {
            include: vec!["**/*".to_string()],
            ..RawConfiguration::default()
        };

        let WalkDirectoryResult {
            included_paths: files,
            included_packs: _,
        } = walk_directory(absolute_path.clone(), &raw_config);

        let node_module_file =
            absolute_path.join("node_modules/subfolder/file.rb");
        let contains_bad_file = files.contains(&node_module_file);
        assert!(!contains_bad_file);

        // Although `node_modules/**/*` should probably exclude `node_modules/file.rb`,
        // it skips the first files in the directory. For now this doesn't affect behavior,
        // in Gusto's monolith, so keeping as an open bug for now.
        // To fix this bug, start by changing this test to:
        // assert!(!contains_bad_file); (instead of assert!(contains_bad_file);)
        let node_module_file = absolute_path.join("node_modules/file.rb");
        let contains_bad_file = files.contains(&node_module_file);
        assert!(contains_bad_file);

        Ok(())
    }
}
