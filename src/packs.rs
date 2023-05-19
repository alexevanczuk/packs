
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
