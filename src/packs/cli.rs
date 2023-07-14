use crate::packs;
use crate::packs::checker;

use crate::packs::file_utils::get_absolute_path;
use clap::{Parser, Subcommand};
use clap_derive::Args;
use std::path::PathBuf;
use tracing::debug;

use super::logger::install_logger;

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

    /// Print to console when files begin and finish processing (to identify files that panic when processing files concurrently)
    #[arg(short, long)]
    print_files: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[clap(about = "Just saying hi")]
    Greet,

    #[clap(about = "Look for violations in the codebase")]
    Check { files: Vec<String> },

    #[clap(about = "Check file contents piped to stdin")]
    CheckContents { file: String },

    #[clap(
        about = "Update package_todo.yml files with the current violations"
    )]
    Update,

    #[clap(about = "Look for validation errors in the codebase")]
    Validate,

    #[clap(
        about = "Check for dependencies that when removed produce no violations."
    )]
    CheckUnnecessaryDependencies,

    #[clap(
        about = "Expose monkey patches of the Ruby stdlib, gems your app uses, and your application itself"
    )]
    ExposeMonkeyPatches(ExposeMonkeyPatchesArgs),

    #[clap(
        about = "Reports a small number of edges that when removed make the graph acyclic."
    )]
    CheckMinEdges,

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
    ListDefinitions(ListDefinitionsArgs),
}

#[derive(Debug, Args)]
struct ListDefinitionsArgs {
    /// Show constants with multiple definitions only
    #[arg(short, long)]
    ambiguous: bool,
}

#[derive(Debug, Args)]
struct ExposeMonkeyPatchesArgs {
    /// An absolute path to the directory containing Ruby source code (for extracting definitions from Ruby stdlib)
    /// Example: /Users/alex.evanczuk/.rbenv/versions/3.2.2/lib/ruby/3.2.0/
    #[arg(short, long)]
    rubydir: PathBuf,

    /// An absolute path to the directory containing your gems (for extracting definitions from gem source code)
    /// Example: /Users/alex.evanczuk/.rbenv/versions/3.2.2/lib/ruby/gems/3.2.0/gems/
    #[arg(short, long)]
    gemdir: PathBuf,
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

    if args.print_files {
        configuration.print_files = true;
    }

    if args.experimental_parser {
        debug!("Using experimental parser");
        configuration.experimental_parser = true;
    }

    if args.no_cache {
        debug!("Cache is disabled");
        configuration.cache_enabled = false
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
        Command::CheckContents { file } => {
            let absolute_path = get_absolute_path(file.clone(), &configuration);
            configuration.stdin_file_path = Some(absolute_path);
            checker::check_all(configuration, vec![file])
        }
        Command::Update => checker::update(configuration),
        Command::Validate => {
            checker::validate_all(&configuration)
            // Err("💡 Please use `packs check` to detect dependency cycles and run other configuration validations".into())
        }
        Command::CheckUnnecessaryDependencies => {
            packs::checker::check_unnecessary_dependencies(&configuration)
        }
        Command::DeleteCache => {
            packs::delete_cache(configuration);
            Ok(())
        }
        Command::ListDefinitions(args) => {
            let ambiguous = args.ambiguous;
            packs::list_definitions(&configuration, ambiguous);
            Ok(())
        }
        Command::ExposeMonkeyPatches(args) => {
            packs::expose_monkey_patches(
                &configuration,
                &args.rubydir,
                &args.gemdir,
            );
            Ok(())
        }
        Command::CheckMinEdges => {
            packs::checker::check_min_edges(&configuration);
            Ok(())
        }
    }
}
