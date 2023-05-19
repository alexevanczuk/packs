
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

pub fn all() -> Vec<Pack> {
    let packs: Vec<Pack> = Vec::new();
    packs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let expected_packs: Vec<Pack> = Vec::new();
        assert_eq!(all(), expected_packs);
    }

}
