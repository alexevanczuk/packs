use crate::packs;
use crate::packs::checker;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use super::logger::install_logger;
use super::parsing::ruby::zeitwerk_utils::get_zeitwerk_constant_resolver;

#[derive(Subcommand, Debug)]
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
        about = "`rm -rf` on your cache directory, default `tmp/cache/packwerk`"
    )]
    DeleteCache,

    #[clap(
        about = "List packs based on configuration in packwerk.yml (for debugging purposes)"
    )]
    ListPacks,

    #[clap(
        about = "List analyzed files based on configuration in packwerk.yml (for debugging purposes)"
    )]
    ListIncludedFiles,

    #[clap(
        about = "List the constants that packs sees and where it sees them (for debugging purposes)"
    )]
    ListDefinitions,
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

    /// Run with performance debug mode
    #[arg(short, long)]
    debug: bool,
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

    install_logger(args.debug);

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
        Command::DeleteCache => {
            packs::delete_cache(configuration);
            Ok(())
        }
        Command::ListDefinitions => {
            // TODO: This and other commands that fetch the constant resolver
            // Should respect the configuration flag.
            let constant_resolver = get_zeitwerk_constant_resolver(
                &configuration.pack_set,
                &absolute_root,
                &configuration.cache_directory,
            );
            packs::list_definitions(&constant_resolver);
            Ok(())
        }
    }
}
