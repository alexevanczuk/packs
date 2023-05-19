
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

#[derive(Debug, PartialEq)]  // Implement PartialEq trait
pub struct Pack {

}

pub fn all(absolute_root: PathBuf) -> Vec<Pack> {
    let mut packs: Vec<Pack> = Vec::new();
    let pattern = absolute_root.join("**/package.yml");
    for entry in glob(pattern.to_str().unwrap()).expect("Failed to read glob pattern") {
        match entry {
            Ok(_path) => packs.push(Pack{}),
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
        expected_packs.push(Pack {});
        expected_packs.push(Pack {});
        expected_packs.push(Pack {});
        let absolute_root = PathBuf::from("tests/fixtures/simple_dependency_violation");
        assert_eq!(all(absolute_root), expected_packs);
    }

}
