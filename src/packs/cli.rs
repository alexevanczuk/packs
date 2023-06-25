use crate::packs;
use crate::packs::checker;
use crate::packs::per_file_cache;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
// We use snake_case as this is currently the conventon for the Ruby ecosystem,
// and this is a Ruby tool (for now!)
#[clap(rename_all = "snake_case")]
enum Command {
    #[clap(about = "Just saying hi")]
    Greet,

    #[clap(about = "Look for violations in the codebase")]
    Check { files: Vec<String> },

    #[clap(
        about = "Update package_todo.yml files with the current violations"
    )]
    Update,

    #[clap(about = "Look for validation errors in the codebase")]
    Validate,

    #[clap(
        about = "Generate a cache to be used by the ruby implementation of packwerk"
    )]
    GenerateCache { files: Vec<String> },

    #[clap(about = "List packs based on configuration in packwerk.yml")]
    ListPacks,

    #[clap(
        about = "`rm -rf` on your cache directory, usually `tmp/cache/packwerk`"
    )]
    DeleteCache,

    #[clap(
        about = "List analyzed files based on configuration in packwerk.yml"
    )]
    ListIncludedFiles,
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

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let absolute_root = args
        .absolute_project_root()
        .expect("Issue getting absolute_project_root!");

    let configuration = packs::configuration::get(&absolute_root);

    match args.command {
        Command::Greet => {
            packs::greet();
            Ok(())
        }
        Command::ListPacks => {
            packs::list(configuration);
            Ok(())
        }
        Command::ListIncludedFiles => {
            configuration
                .included_files
                .iter()
                .for_each(|f| println!("{}", f.display()));
            Ok(())
        }
        Command::Check { files } => checker::check(configuration, files),
        Command::Update => checker::update(configuration),
        Command::Validate => Err("💡 This command is coming soon!".into()),
        Command::GenerateCache { files } => {
            if !configuration.cache_enabled {
                return Err("Cache is disabled. Enable it in packwerk.yml to use this command.".into());
            }

            per_file_cache::write_cache_for_files(files, configuration);
            Ok(())
        }
        Command::DeleteCache => {
            packs::delete_cache(configuration);
            Ok(())
        }
    }
}
