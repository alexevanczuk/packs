use glob::glob;
use std::path::PathBuf;

mod cache;
pub(crate) mod cli;
pub mod parser;
mod string_helpers;

// Re-exports: Eventually, these may be part of the public API for packs
pub use parser::ruby::packwerk::extractor::Range;
pub use parser::ruby::packwerk::extractor::Reference;

pub fn greet() {
    println!("Hello! This CLI is under construction.")
}

pub fn list(absolute_root: PathBuf) {
    for pack in all(absolute_root) {
        println!("{}", pack.yml.display())
    }
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd)] // Implement PartialEq trait
pub struct Pack {
    yml: PathBuf,
    name: String,
}

impl Pack {
    pub fn from_path(absolute_root: &PathBuf, yml: PathBuf) -> Pack {
        let mut name = yml
            .strip_prefix(absolute_root)
            .expect("Absolute root is not a prefix to pack YML – should not happen!")
            .parent()
            .expect("Expected package to be in a parent directory")
            .to_str()
            .expect("Non-unicode characters?")
            .to_owned();

        if name == *"" {
            name = String::from(".")
        }

        Pack { yml, name }
    }
}

pub fn all(absolute_root: PathBuf) -> Vec<Pack> {
    let mut packs: Vec<Pack> = Vec::new();
    let pattern = absolute_root.join("**/package.yml");
    for entry in
        glob(pattern.to_str().unwrap()).expect("Failed to read glob pattern")
    {
        match entry {
            Ok(yml) => packs.push(Pack::from_path(&absolute_root, yml)),
            Err(e) => println!("{:?}", e),
        }
    }

    packs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all() {
        let mut expected_packs: Vec<Pack> = Vec::new();
        let absolute_root: PathBuf = PathBuf::from("tests/fixtures/simple_app");

        let foo_yml =
            absolute_root.join(PathBuf::from("packs/foo/package.yml"));
        let root_yml = absolute_root.join(PathBuf::from("package.yml"));
        let bar_yml =
            absolute_root.join(PathBuf::from("packs/bar/package.yml"));
        expected_packs.push(Pack {
            yml: foo_yml,
            name: String::from("packs/foo"),
        });
        expected_packs.push(Pack {
            yml: root_yml,
            name: String::from("."),
        });
        expected_packs.push(Pack {
            yml: bar_yml,
            name: String::from("packs/bar"),
        });

        let mut actual = all(absolute_root);
        actual.sort();
        expected_packs.sort();
        assert_eq!(actual, expected_packs);
    }
}
