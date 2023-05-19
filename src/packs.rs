use glob::glob;
use std::path::PathBuf;

pub fn greet() -> () {
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
}

impl Pack {
    pub fn from(yml: PathBuf) -> Pack {
        Pack { yml }
    }
}

pub fn all(absolute_root: PathBuf) -> Vec<Pack> {
    let mut packs: Vec<Pack> = Vec::new();
    let pattern = absolute_root.join("**/package.yml");
    for entry in glob(pattern.to_str().unwrap()).expect("Failed to read glob pattern") {
        match entry {
            Ok(yml) => packs.push(Pack { yml }),
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
        expected_packs.push(Pack::from(foo_yml));
        expected_packs.push(Pack::from(root_yml));
        expected_packs.push(Pack::from(bar_yml));

        assert_eq!(all(absolute_root).sort(), expected_packs.sort());
    }
}
