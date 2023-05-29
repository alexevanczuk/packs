use crate::packs::cache::write_cache;
use crate::packs::parser;
use crate::packs::{self, string_helpers};
use clap::{Parser, Subcommand};
use glob::glob;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

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
            parser::get_references(&absolute_root);
        }
        Command::GenerateCache { mut files } => {
            let pattern = absolute_root.join("app/**/*.rb");

            if files.len() == 0 {
                let paths =
                    files.into_iter().map(|s| PathBuf::from(s)).collect();
            } else {
                let paths = glob(pattern.to_str().unwrap())
                    .expect("Failed to read glob pattern");
            }

            paths
                .par_bridge() // Parallel iterator
                .for_each(|entry| match entry {
                    Ok(path) => {
                        write_cache(absolute_root.as_path(), path.as_path())
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        panic!("blah");
                    }
                });
        }
    }
}
