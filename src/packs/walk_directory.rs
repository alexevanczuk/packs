use jwalk::WalkDirGeneric;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};
use tracing::debug;

use super::{
    file_utils::build_glob_set, pack::Pack, raw_configuration::RawConfiguration,
};

pub struct WalkDirectoryResult {
    pub included_files: HashSet<PathBuf>,
    pub included_packs: HashSet<Pack>,
    pub owning_package_yml_for_file: HashMap<PathBuf, PathBuf>,
}

#[derive(Debug, Default, Clone)]
struct ProcessReadDirState {
    current_package_yml: PathBuf,
}

impl jwalk::ClientState for ProcessReadDirState {
    type ReadDirState = ProcessReadDirState;

    type DirEntryState = ProcessReadDirState;
}

// We use jwalk to walk directories in parallel and compare them to the `include` and `exclude` patterns
// specified in the `RawConfiguration`
// https://docs.rs/jwalk/0.8.1/jwalk/struct.WalkDirGeneric.html#method.process_read_dir
// We only walk the directory once and pull all of the information we need from it,
// which is faster than walking the directory multiple times.
// Likely, we can organize this better by moving each piece of logic into its own function so this function
// allows for a sort of "visitor pattern" for different things that need to walk the directory.
pub(crate) fn walk_directory(
    absolute_root: PathBuf,
    raw: &RawConfiguration,
) -> anyhow::Result<WalkDirectoryResult> {
    debug!("Beginning directory walk");

    let mut included_files: HashSet<PathBuf> = HashSet::new();
    let mut included_packs: HashSet<Pack> = HashSet::new();
    let mut owning_package_yml_for_file: HashMap<PathBuf, PathBuf> =
        HashMap::new();

    // Create this vector outside of the closure to avoid reallocating it
    let default_excluded_dirs = [
        "node_modules/**/*",
        "vendor/**/*",
        "tmp/**/*",
        ".git/**/*",
        "public/**/*",
        "bin/**/*",
        "log/**/*",
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
    let current_package_yml = PathBuf::from("package.yml");

    let walk_dir = WalkDirGeneric::<ProcessReadDirState>::new(&absolute_root)
        .follow_links(true)
        .root_read_dir_state(ProcessReadDirState {
            current_package_yml,
        })
        .process_read_dir(
            move |_depth, absolute_dirname, read_dir_state, children| {
                // We need to let the compiler know that we are using a reference and not the value itself.
                // We need to then clone the Arc to get a new reference, which is a new pointer to the value/data
                // (with an increase to the reference count).
                let cloned_excluded_dirs = excluded_dirs_ref.clone();
                let cloned_absolute_root = absolute_root_ref.clone();
                let package_yml = absolute_dirname.join("package.yml");

                // Even if the parent has set this on children, the existence of a new
                // package.yml file should override it.
                if package_yml.exists() {
                    read_dir_state.current_package_yml = package_yml;
                }

                children.iter_mut().for_each(|child_dir_entry_result| {
                    if let Ok(child_dir_entry) = child_dir_entry_result {
                        let child_absolute_dirname = child_dir_entry.path();
                        child_dir_entry
                            .client_state
                            .current_package_yml
                            .clone_from(&read_dir_state.current_package_yml);

                        let relative_path = child_absolute_dirname
                            .strip_prefix(cloned_absolute_root.as_ref())
                            .unwrap();
                        if cloned_excluded_dirs.as_ref().is_match(relative_path)
                        {
                            child_dir_entry.read_children_path = None;
                        }
                    }
                });
            },
        );

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

        let unwrapped_entry = entry;
        if let Err(_e) = unwrapped_entry {
            // Encountered an invalid symlink. Being consistent with packwerk, which swallows this error and continues
            continue;
        }
        let unwrapped_entry = unwrapped_entry.unwrap();

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

        let current_package_yml =
            &unwrapped_entry.client_state.current_package_yml;

        if &absolute_path == current_package_yml
            // Ideally, we don't need the second part of this conditional, but it's here
            // because there is a bug where the root pack doesn't match package_paths.
            // We know we always want the root pack to be registered, since it's the catch-all pack for
            // where constants are defined if they are not in another pack.
            // We can remove this once we fix the bug.
            && (package_paths_set.is_match(relative_path.parent().unwrap()) || absolute_path.parent().unwrap() == absolute_root)
        {
            let pack = Pack::from_path(&absolute_path, &absolute_root)?;
            included_packs.insert(pack);
        }

        // This could be one line, but I'm keeping it separate for debugging purposes
        if includes_set.is_match(&relative_path) {
            if !excludes_set.is_match(&relative_path) {
                included_files.insert(absolute_path.clone());
                owning_package_yml_for_file
                    .insert(absolute_path, current_package_yml.clone());
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
    }

    debug!("Finished directory walk");

    Ok(WalkDirectoryResult {
        included_files,
        included_packs,
        owning_package_yml_for_file,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::packs::{
        raw_configuration::RawConfiguration, walk_directory::walk_directory,
    };

    #[test]
    fn test_walk_directory() -> anyhow::Result<()> {
        let absolute_path = PathBuf::from("tests/fixtures/simple_app")
            .canonicalize()
            .expect("Could not canonicalize path");

        let raw_config = RawConfiguration {
            include: vec!["**/*".to_string()],
            ..RawConfiguration::default()
        };

        let walk_directory_result =
            walk_directory(absolute_path.clone(), &raw_config);
        assert!(walk_directory_result.is_ok());
        let included_files = walk_directory_result?.included_files;

        let node_module_file =
            absolute_path.join("node_modules/subfolder/file.rb");
        let contains_bad_file = included_files.contains(&node_module_file);
        assert!(!contains_bad_file);

        let node_module_file = absolute_path.join("node_modules/file.rb");
        let contains_bad_file = included_files.contains(&node_module_file);
        assert!(!contains_bad_file);

        Ok(())
    }
}
