use itertools::Itertools;
use jwalk::WalkDirGeneric;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::File,
    path::{Path, PathBuf},
};

use crate::packs::Pack;

// See: Setting up the configuration file
// https://github.com/Shopify/packwerk/blob/main/USAGE.md#setting-up-the-configuration-file
#[derive(Debug, Deserialize, Serialize, Default)]
struct RawConfiguration {
    // List of patterns for folder paths to include
    #[serde(default = "default_include")]
    include: Vec<String>,

    // List of patterns for folder paths to exclude
    #[serde(default = "default_exclude")]
    exclude: Vec<String>,

    // Patterns to find package configuration files
    #[serde(default = "default_package_paths")]
    package_paths: Vec<String>,

    // List of custom associations, if any
    #[serde(default = "default_custom_associations")]
    custom_associations: Vec<String>,

    // Whether or not you want the cache enabled
    #[serde(default = "default_cache")]
    cache: bool,

    // Where you want the cache to be stored
    #[serde(default = "default_cache_directory")]
    cache_directory: String,
}

fn default_include() -> Vec<String> {
    vec![
        String::from("**/*.rb"),
        String::from("**/*.rake"),
        String::from("**/*.erb"),
    ]
}

fn default_exclude() -> Vec<String> {
    vec![String::from("{bin,node_modules,script,tmp,vendor}/**/*")]
}

fn default_package_paths() -> Vec<String> {
    vec![String::from("**/*")]
}

fn default_custom_associations() -> Vec<String> {
    vec![]
}

fn default_cache() -> bool {
    true
}

fn default_cache_directory() -> String {
    String::from("tmp/cache/packwerk")
}

#[derive(Default)]
pub struct Configuration {
    pub included_files: HashSet<PathBuf>,
    pub absolute_root: PathBuf,
    pub packs: Vec<Pack>,
    pub cache_enabled: bool,
    pub cache_directory: PathBuf,
}

impl Configuration {
    pub(crate) fn intersect_files(
        &self,
        input_files: Vec<String>,
    ) -> HashSet<PathBuf> {
        if input_files.is_empty() {
            self.included_files.clone()
        } else {
            let input_paths = input_files
                .iter()
                .map(PathBuf::from)
                .flat_map(|p| {
                    if p.is_absolute() {
                        vec![p]
                    } else {
                        let absolute_path = self.absolute_root.join(&p);
                        if absolute_path.is_dir() {
                            glob::glob(
                                absolute_path.join("**/*.*").to_str().unwrap(),
                            )
                            .expect("Failed to read glob pattern")
                            .filter_map(Result::ok)
                            .collect::<Vec<_>>()
                        } else {
                            vec![absolute_path]
                        }
                    }
                })
                .collect::<HashSet<_>>();

            self.included_files
                .intersection(&input_paths)
                .cloned()
                .collect::<HashSet<PathBuf>>()
        }
    }
}

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
fn walk_directory(
    absolute_root: &Path,
    raw: &RawConfiguration,
) -> (HashSet<PathBuf>, HashSet<Pack>) {
    let mut included_paths: HashSet<PathBuf> = HashSet::new();
    let mut included_packs: HashSet<Pack> = HashSet::new();

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
        .process_read_dir(|depth, _path, _read_dir_state, children| {
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
                    //     .strip_prefix(&*shared_path_clone)
                    //     .unwrap();

                    // if matches_globs(&relative_path, &raw.exclude) {
                    //     dir_entry.read_children_path = None;
                    // }

                    // So instead, we'll just hardcode the directories we want to exclude
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

                    let dirname = dir_entry.path();
                    for excluded_dir in excluded_dirs {
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

        let relative_path = absolute_path
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

        if file_name.eq_ignore_ascii_case("package.yml")
            && matches_globs(
                relative_path.parent().unwrap(),
                &raw.package_paths,
            )
        {
            //
            // Soon I'll be actually deserializing the package.yml file
            // to grab things like enforce_dependencies.
            // For now, we just construct the pack.
            // We should consider if checkers and such can add their own deserialization
            // behavior to Pack via a trait or something? That would make extension simpler!
            //
            // let file = File::open(&absolute_path).unwrap_or_else(|_| {
            //     panic!("Could not open {}", &absolute_path.display())
            // });
            // let deserialized_pack: Option<Pack> = serde_yaml::from_reader(file)
            //     .unwrap_or_else(|_| {
            //         panic!("Could not parse {}", &absolute_path.display())
            //     });

            let mut name = relative_path
                .parent()
                .expect("Expected package to be in a parent directory")
                .to_str()
                .expect("Non-unicode characters?")
                .to_owned();
            let yml = absolute_path.clone();
            // Handle the root pack
            if name == *"" {
                name = String::from(".")
            };
            let pack: Pack = Pack { yml, name };
            included_packs.insert(pack);
        }
    }

    (included_paths, included_packs)
}

pub(crate) fn get(absolute_root: &Path) -> Configuration {
    let absolute_path_to_packwerk_yml = absolute_root.join("packwerk.yml");

    let raw_config: RawConfiguration =
        if absolute_path_to_packwerk_yml.exists() {
            let mut file = File::open(absolute_path_to_packwerk_yml.clone())
                .unwrap_or_else(|e| {
                    panic!(
                        "Could not open packwerk.yml at: {} due to error: {:?}",
                        absolute_path_to_packwerk_yml.display(),
                        e
                    )
                });

            let mut contents = String::new();
            std::io::Read::read_to_string(&mut file, &mut contents)
                .unwrap_or_else(|e| {
                    panic!(
                        "Could not read packwerk.yml at: {} due to error: {:?}",
                        absolute_path_to_packwerk_yml.display(),
                        e
                    )
                });

            serde_yaml::from_str(&contents).unwrap_or_else(|e| {
                panic!(
                    "Could not parse packwerk.yml at: {} due to error: {:?}",
                    absolute_path_to_packwerk_yml.display(),
                    e
                )
            })
        } else {
            RawConfiguration::default()
        };

    let (included_files, unsorted_packs) =
        walk_directory(absolute_root, &raw_config);
    let packs = unsorted_packs
        .into_iter()
        .sorted_by(|packa, packb| {
            Ord::cmp(&packb.name.len(), &packa.name.len())
                .then_with(|| packa.name.cmp(&packb.name))
        })
        .collect();

    Configuration {
        included_files,
        absolute_root: absolute_root.to_path_buf(),
        packs,
        cache_enabled: raw_config.cache,
        cache_directory: absolute_root.join(raw_config.cache_directory),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::packs::configuration;

    #[test]
    fn default_options() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let actual = configuration::get(&absolute_root);
        assert_eq!(actual.absolute_root, absolute_root);

        let expected_included_files = vec![
            absolute_root.join("packs/bar/app/services/bar.rb"),
            absolute_root.join("packs/foo/app/services/foo.rb"),
            absolute_root.join("packs/foo/app/views/foo.erb"),
            absolute_root.join("packs/baz/app/services/baz.rb"),
        ]
        .into_iter()
        .collect::<HashSet<PathBuf>>();
        assert_eq!(actual.included_files, expected_included_files);

        let expected_packs = vec![
            Pack {
                yml: absolute_root.join("packs/bar/package.yml"),
                name: String::from("packs/bar"),
            },
            Pack {
                yml: absolute_root.join("packs/baz/package.yml"),
                name: String::from("packs/baz"),
            },
            Pack {
                yml: absolute_root.join("packs/foo/package.yml"),
                name: String::from("packs/foo"),
            },
            Pack {
                yml: absolute_root.join("package.yml"),
                name: String::from("."),
            },
        ];

        assert_eq!(expected_packs, actual.packs);

        assert!(actual.cache_enabled)
    }

    #[test]
    fn filtered_absolute_paths_with_nonempty_input_paths() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let configuration = configuration::get(&absolute_root);
        let actual_paths = configuration.intersect_files(vec![
            String::from("packs/foo/app/services/foo.rb"),
            String::from("scripts/my_script.rb"),
            String::from("packs/bar/app/services/bar.rb"),
            String::from("vendor/some_gem/foo.rb"),
        ]);
        let expected_paths = vec![
            absolute_root.join("packs/bar/app/services/bar.rb"),
            absolute_root.join("packs/foo/app/services/foo.rb"),
        ]
        .into_iter()
        .collect::<HashSet<PathBuf>>();
        assert_eq!(actual_paths, expected_paths);
    }

    #[test]
    fn filtered_absolute_paths_with_empty_input_paths() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let configuration = configuration::get(&absolute_root);
        let actual_paths = configuration.intersect_files(vec![]);
        let expected_paths = vec![
            absolute_root.join("packs/bar/app/services/bar.rb"),
            absolute_root.join("packs/foo/app/services/foo.rb"),
            absolute_root.join("packs/foo/app/views/foo.erb"),
            absolute_root.join("packs/baz/app/services/baz.rb"),
        ]
        .into_iter()
        .collect::<HashSet<PathBuf>>();
        assert_eq!(actual_paths, expected_paths);
    }

    #[test]
    fn filtered_absolute_paths_with_directory_input_paths() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let configuration = configuration::get(&absolute_root);
        let actual_paths =
            configuration.intersect_files(vec![String::from("packs/bar")]);
        let expected_paths =
            vec![absolute_root.join("packs/bar/app/services/bar.rb")]
                .into_iter()
                .collect::<HashSet<PathBuf>>();
        assert_eq!(actual_paths, expected_paths);
    }
}
