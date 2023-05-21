use glob::glob;
use std::path::PathBuf;

pub mod parser;

pub fn greet() {
    println!("Hello! This CLI is under construction.")
}

pub fn list(absolute_root: PathBuf) {
    let pattern = absolute_root.join("packs/*/package.yml");
    for entry in glob(pattern.to_str().unwrap()).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => println!("{:?}", path.display()),
            Err(e) => println!("{:?}", e),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd)] // Implement PartialEq trait
pub struct Pack {
    yml: PathBuf,
    name: String,
}

impl Pack {
    pub fn from(absolute_root: &PathBuf, yml: PathBuf) -> Pack {
        let name = yml
            .strip_prefix(absolute_root)
            .expect("Absolute root is not a prefix to pack YML – should not happen!")
            .to_str()
            .expect("Non-unicode characters?")
            .to_owned();
        Pack { yml, name }
    }
}

pub fn all(absolute_root: PathBuf) -> Vec<Pack> {
    let mut packs: Vec<Pack> = Vec::new();
    let pattern = absolute_root.join("**/package.yml");
    for entry in glob(pattern.to_str().unwrap()).expect("Failed to read glob pattern") {
        match entry {
            Ok(yml) => packs.push(Pack::from(&absolute_root, yml)),
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
        let absolute_root: PathBuf = PathBuf::from("tests/fixtures/simple_dependency_violation");

        let foo_yml = absolute_root.join(PathBuf::from("packs/foo/package.yml"));
        let root_yml = absolute_root.join(PathBuf::from("package.yml"));
        let bar_yml = absolute_root.join(PathBuf::from("packs/bar/package.yml"));
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

        assert_eq!(all(absolute_root).sort(), expected_packs.sort());
    }
}
