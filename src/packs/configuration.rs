use std::path::PathBuf;

use itertools::Itertools;

pub struct Configuration {
    pub included_files: Vec<PathBuf>,
    pub absolute_root: PathBuf,
}
impl Configuration {
    fn default(absolute_root: PathBuf) -> Configuration {
        let default_include_patterns = vec![String::from("**/*.{rb,rake,erb}")];
        let default_exclude_patterns =
            vec![String::from("{bin,node_modules,script,tmp,vendor}/**/*")];

        // Adding a `!` to the beginning of a glob pattern negates it.
        let exclude_patterns =
            default_exclude_patterns.iter().map(|p| format!("!{}", p));

        let mut combined_patterns = default_include_patterns;
        combined_patterns.extend(exclude_patterns);

        let included_files: Vec<PathBuf> =
            globwalk::GlobWalkerBuilder::from_patterns(
                absolute_root.clone(),
                &combined_patterns,
            )
            .build()
            .expect("Could not build glob walker")
            .filter_map(Result::ok)
            .map(|x| x.into_path())
            .sorted() // Make output deterministic
            .collect();

        Configuration {
            included_files,
            absolute_root,
        }
    }
}

pub(crate) fn get(absolute_root: PathBuf) -> Configuration {
    Configuration::default(absolute_root)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::packs::configuration;

    #[test]
    fn default_options() {
        let absolute_root = PathBuf::from("tests/fixtures/simple_app");
        let actual = configuration::get(absolute_root.clone());
        assert_eq!(actual.absolute_root, absolute_root);
        assert_eq!(
            actual.included_files,
            vec![
                absolute_root.join("packs/bar/app/services/bar.rb"),
                absolute_root.join("packs/foo/app/services/foo.rb"),
                absolute_root.join("packs/foo/app/views/foo.erb")
            ]
        )
    }
}
