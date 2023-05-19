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

// impl PartialOrd for Pack {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         self.yml.partial_cmp(&other.yml)
//     }
// }

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
    fn test_add() {
        let mut expected_packs: Vec<Pack> = Vec::new();
        let absolute_root: PathBuf = PathBuf::from("tests/fixtures/simple_dependency_violation");

        expected_packs.push(Pack {
            yml: absolute_root.join(PathBuf::from("packs/foo/package.yml")),
        });
        expected_packs.push(Pack {
            yml: absolute_root.join(PathBuf::from("package.yml")),
        });
        expected_packs.push(Pack {
            yml: absolute_root.join(PathBuf::from("packs/bar/package.yml")),
        });

        assert_eq!(all(absolute_root).sort(), expected_packs.sort());
    }
}
