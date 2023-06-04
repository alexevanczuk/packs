use std::path::PathBuf;

use itertools::Itertools;

pub struct Configuration {
    pub included_files: Vec<PathBuf>,
    pub absolute_root: PathBuf,
}
impl Configuration {
    fn default(absolute_root: PathBuf) -> Configuration {
        let default_include_patterns = &["**/*.{rb,rake,erb}"];
        let default_exclude_patterns =
            &["{bin,node_modules,script,tmp,vendor}/**/*"];

        let mut patterns = vec![];
        patterns.extend_from_slice(default_include_patterns);
        patterns.extend_from_slice(
            default_exclude_patterns
                .iter()
                .map(|pattern| format!("!{}", pattern).as_str())
                .collect::<&str>()[..],
        );

        let included_files: Vec<PathBuf> =
            globwalk::GlobWalkerBuilder::from_patterns(
                absolute_root.clone(),
                // &["*.{png,jpg,gif}", "!Pictures/*"],
                // default_include_patterns,
                &patterns,
            )
            .build()
            .expect("Could not build glob walker")
            .filter_map(Result::ok)
            .map(|x| x.into_path())
            .sorted()
            .collect();

        // let include = globwalk::glob(default_include_pattern.to_str().unwrap())
        //     .expect("Failed to read glob pattern");

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
                absolute_root.join("packs/foo/app/services/foo.rb")
            ]
        )
    }
}
