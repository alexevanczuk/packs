use crate::packs;
use crate::packs::{cache, parser};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
// We use snake_case as this is currently the conventon for the Ruby ecosystem,
// and this is a Ruby tool (for now!)
#[clap(rename_all = "snake_case")]
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
        Command::Greet => packs::greet(),
        Command::ListPacks => packs::list(absolute_root),
        Command::Check => {
            parser::ruby::packwerk::get_references(&absolute_root);
        }
        Command::GenerateCache { files } => {
            let configuration = packs::configuration::get(absolute_root);
            if !configuration.cache_enabled {
                // TODO: Figure out the idiomatic way to raise tis error
                panic!("Cache is disabled. Enable it in packwerk.yml to use this command.");
            }
            cache::write_cache_for_files(files, configuration);
        }
    }
}
