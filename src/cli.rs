use crate::packs;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
enum Command {
    Greet,
    ListPacks,
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

pub fn cli() {
    let args = Args::parse();
    let absolute_root = args.absolute_project_root().expect("Issue getting absolute_project_root!");
    match args.command {
        Command::Greet => {
            packs::greet();
        }
        Command::ListPacks => packs::list(absolute_root),
    }
}
