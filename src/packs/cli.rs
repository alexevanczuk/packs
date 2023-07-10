use crate::packs;
use crate::packs::checker;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::debug;

use super::logger::install_logger;

use super::Configuration;

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

    /// Run with the experimental parser, which gets constant definitions directly from the AST
    #[arg(short, long)]
    experimental_parser: bool,

    /// Run without the cache (good for CI, testing)
    #[arg(long)]
    no_cache: bool,
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

    let mut configuration = packs::configuration::get(&absolute_root);

    if args.experimental_parser {
        debug!("Using experimental parser");
        configuration = configuration.with_experimental_parser();
    }

    if args.no_cache {
        debug!("Cache is disabled");
        configuration = Configuration {
            cache_enabled: false,
            ..configuration
        };
    }

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
        Command::Check { files } => checker::check_all(configuration, files),
        Command::Update => checker::update(configuration),
        Command::Validate => {
            checker::validate_all(&configuration)
            // Err("💡 Please use `packs check` to detect dependency cycles and run other configuration validations".into())
        }
        Command::DeleteCache => {
            packs::delete_cache(configuration);
            Ok(())
        }
        Command::ListDefinitions => {
            packs::list_definitions(&configuration);
            Ok(())
        }
    }
}
