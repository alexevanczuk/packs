use crate::packs::cache::file_content_digest;
use crate::packs::parser;
use crate::packs::{self, string_helpers};

// Make this import work
// I'm getting this error:
// unresolved import `crate::string_helpers`
// no `string_helpers` in the root
// use crate::string_helpers;

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
enum Command {
    Greet,
    ListPacks,
    Check,
    GenerateCache {
        #[clap(required = true)]
        files: Vec<String>,
    },
}

/// A CLI to interact with packs
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Path for the root of the project
    #[arg(long, default_value = ".")]
    project_root: PathBuf,
}

impl Args {
    fn absolute_project_root(&self) -> Result<PathBuf, std::io::Error> {
        self.project_root.canonicalize()
    }

    // fn absolute_path(&self, path: &Path) -> Result<PathBuf, std::io::Error> {
    //     Ok(self.absolute_project_root()?.join(path))
    // }
}

pub fn run() {
    let args = Args::parse();
    let absolute_root = args
        .absolute_project_root()
        .expect("Issue getting absolute_project_root!");
    match args.command {
        Command::Greet => {
            packs::greet();
        }
        Command::ListPacks => packs::list(absolute_root),
        Command::Check => {
            parser::get_references(absolute_root);
        }
        Command::GenerateCache { mut files } => {
            let file_string = string_helpers::to_sentence(&files);
            println!("Cache was generated for files {}", file_string);
            let mut file_content_digests = HashMap::new();
            files.sort();
            for file in files {
                let path = PathBuf::from(file);
                let absolute_path = absolute_root.join(&path);
                let digest = file_content_digest(&absolute_path);
                file_content_digests.insert(path, digest);
            }
            println!("The file content digests are {:?}", file_content_digests);
        }
    }
}
