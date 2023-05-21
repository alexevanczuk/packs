use glob::glob;
use lib_ruby_parser::{ParserOptions, Parser};
use rayon::prelude::*;
use std::{path::PathBuf, fs, collections::HashMap};

pub fn get_references<'a>(absolute_root: PathBuf) -> HashMap<PathBuf, Vec<&'a str>> {
    // Later this can come from config
    let references_by_file: HashMap<PathBuf, Vec<&str>> = HashMap::new();
    let pattern = absolute_root.join("packs/**/*.rb");

    glob(pattern.to_str().unwrap()).expect("Failed to read glob pattern")
        .par_bridge() // Parallel iterator
        .for_each(|entry| {
            match entry {
                Ok(path) => {
                    let options = ParserOptions {
                        buffer_name: "".to_string(),
                        ..Default::default()
                    };
                    // TODO: This can be a debug statement instead of a print
                    // println!("Now parsing {:?}", path);
                    let contents = fs::read_to_string(&path).expect(&format!("Failed to read contents of {}", path.to_string_lossy()));
                    let parser = Parser::new(contents, options);
                    let _ret = parser.do_parse();
                    // let references = vec!["test"];
                    // references_by_file.insert(path, references);
                }
                Err(e) => println!("{:?}", e),
            }
    });

    // println!("{:#?}", references_by_file);
    references_by_file
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_references() {
        let absolute_root: PathBuf = PathBuf::from("tests/fixtures/simple_dependency_violation");
        let references_by_file: HashMap<PathBuf, Vec<&str>> = HashMap::new();

        assert_eq!(get_references(absolute_root), references_by_file);
        // panic!("for output...")
    }
}
