use jwalk::WalkDirGeneric;
use std::{
    collections::HashSet,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use super::{configuration::RawConfiguration, CheckerSetting, PackageTodo};
use crate::packs::Pack;
use crate::packs::RawPack;

fn matches_globs(path: &Path, globs: &[String]) -> bool {
    globs
        .iter()
        .any(|glob| glob_match::glob_match(glob, path.to_str().unwrap()))
}

// We use jwalk to walk directories in parallel and compare them to the `include` and `exclude` patterns
// specified in the `RawConfiguration`
// https://docs.rs/jwalk/0.8.1/jwalk/struct.WalkDirGeneric.html#method.process_read_dir
// We only walk the directory once and pull all of the information we need from it,
// which is faster than walking the directory multiple times.
// Likely, we can organize this better by moving each piece of logic into its own function so this function
// allows for a sort of "visitor pattern" for different things that need to walk the directory.
pub fn walk_directory(
    absolute_root: &Path,
    raw: &RawConfiguration,
) -> (HashSet<PathBuf>, HashSet<Pack>) {
    let mut included_paths: HashSet<PathBuf> = HashSet::new();
    let mut included_packs: HashSet<Pack> = HashSet::new();
    // Create this vector outside of the closure to avoid reallocating it
    let excluded_dirs = vec![
        "node_modules",
        "vendor",
        "tmp",
        ".git",
        "public",
        "bin",
        "log",
        "frontend",
        "sorbet",
    ];

    let excluded_dirs_ref = Arc::new(excluded_dirs);

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
    let walk_dir = WalkDirGeneric::<(usize, bool)>::new(absolute_root)
        .process_read_dir(move |depth, _path, _read_dir_state, children| {
            // We need to let the compiler know that we are using a reference and not the value itself.
            // We need to then clone the Arc to get a new reference, which is a new pointer to the value/data
            // (with an increase to the reference count).
            let cloned_excluded_dirs = excluded_dirs_ref.clone();

            // Excluded dirs are top-level only
            if let Some(depth) = depth {
                if depth > 2 {
                    return;
                }
            }
            children.iter_mut().for_each(|dir_entry_result| {
                if let Ok(dir_entry) = dir_entry_result {
                    // Can't figure out how to actually match against raw.exclude due to ownership issues
                    // Hope to learn soon!
                    // let absolute_path = dir_entry_result.unwrap().path();
                    // let relative_path = absolute_path
                    //     .strip_prefix(&absolute_root)
                    //     .unwrap();

                    // if matches_globs(&relative_path, &raw.exclude) {
                    //     dir_entry.read_children_path = None;
                    // }

                    // So instead, we'll just use the hardcoded directories we want to exclude
                    let dirname = dir_entry.path();
                    for excluded_dir in cloned_excluded_dirs.iter() {
                        if dirname.ends_with(excluded_dir) {
                            dir_entry.read_children_path = None;
                            break;
                        }
                    }
                }
            });
        });

    for entry in walk_dir {
        // I was using this to explore what directories were being walked to potentially
        // find performance improvements.
        // use std::io::Write;
        // // Write the entry out to a log file:
        // let mut file = std::fs::OpenOptions::new()
        //     .create(true)
        //     .append(true)
        //     .open("tmp/pks_log.txt")
        //     .unwrap();
        // writeln!(file, "{:?}", entry).unwrap();
        let absolute_path = entry.unwrap().path();

        if absolute_path.is_dir() {
            continue;
        }

        let mut relative_path = absolute_path
            .strip_prefix(absolute_root)
            .unwrap()
            .to_owned();

        if matches_globs(&relative_path, &raw.include)
            && !matches_globs(&relative_path, &raw.exclude)
        {
            included_paths.insert(absolute_path.clone());
        }

        let file_name =
            relative_path.file_name().expect("expected a file_name");

        // TODO: Move this whole case statement into some function within Pack
        if file_name.eq_ignore_ascii_case("package.yml")
            && (matches_globs(
                relative_path.parent().unwrap(),
                &raw.package_paths,
            ) || absolute_path.parent().unwrap() == absolute_root)
        {
            let mut name = relative_path
                .parent()
                .expect("Expected package to be in a parent directory")
                .to_str()
                .expect("Non-unicode characters?")
                .to_owned();
            let yml = absolute_path.clone();
            relative_path = relative_path.parent().unwrap().to_owned();

            // Handle the root pack
            if name == *"" {
                name = String::from(".");
                relative_path = PathBuf::from(".");
            };

            let mut yaml_contents = String::new();
            let mut file =
                File::open(&yml).expect("Failed to open the YAML file");
            file.read_to_string(&mut yaml_contents)
                .expect("Failed to read the YAML file");

            let raw_pack: RawPack = serde_yaml::from_str(&yaml_contents)
                .expect("Failed to deserialize the YAML");

            let absolute_path_to_package_todo =
                absolute_path.parent().unwrap().join("package_todo.yml");

            let package_todo: PackageTodo =
                if absolute_path_to_package_todo.exists() {
                    let mut package_todo_contents = String::new();
                    let mut file = File::open(&absolute_path_to_package_todo)
                        .expect("Failed to open the package_todo.yml file");
                    file.read_to_string(&mut package_todo_contents)
                        .expect("Could not read the package_todo.yml file");
                    serde_yaml::from_str(&package_todo_contents).unwrap()
                } else {
                    PackageTodo::default()
                };

            let dependencies = raw_pack.dependencies;
            let ignored_dependencies = raw_pack.ignored_dependencies;
            let raw_enforce_dependencies = raw_pack.enforce_dependencies;
            let enforce_dependencies = if raw_enforce_dependencies == "true" {
                CheckerSetting::True
            } else if raw_enforce_dependencies == "strict" {
                CheckerSetting::Strict
            } else {
                CheckerSetting::False
            };

            let pack: Pack = Pack {
                yml,
                name,
                relative_path,
                dependencies,
                package_todo,
                ignored_dependencies,
                enforce_dependencies,
            };
            included_packs.insert(pack);
        }
    }

    (included_paths, included_packs)
}
