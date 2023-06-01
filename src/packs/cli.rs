use crate::packs::cache::write_cache;
use crate::packs::parser;
use crate::packs::{self};
use clap::{Parser, Subcommand};
use glob::glob;
use rayon::prelude::*;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
enum Command {
    Greet,
    ListPacks,
    Check,
    GenerateCache { files: Vec<String> },
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
        Command::GenerateCache { files } => {
            // TODO: This needs to parse include and exclude paths from packwerk.yml
            // to generate a more accurate cache. Could just use default packs ones to start?
            if !files.is_empty() {
                files.into_iter().par_bridge().for_each(|file| {
                    let path = PathBuf::from(file);
                    write_cache(absolute_root.as_path(), path.as_path())
                })
            } else {
                let pattern = absolute_root.join("packs/**/*.rb");
                let paths = glob(pattern.to_str().unwrap())
                    .expect("Failed to read glob pattern");
                paths.par_bridge().for_each(|path| match path {
                    Ok(path) => {
                        let relative_path =
                            path.strip_prefix(absolute_root.as_path()).unwrap();
                        write_cache(absolute_root.as_path(), relative_path);
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        panic!("blah");
                    }
                });
            }
        }
    }
}
