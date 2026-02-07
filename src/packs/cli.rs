use crate::packs;

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

    /// Globally disable enforce_dependency
    #[arg(long)]
    disable_enforce_dependencies: bool,

    /// Globally disable enforce_folder_privacy
    #[arg(long)]
    disable_enforce_folder_privacy: bool,

    /// Globally disable enforce_layers
    #[arg(long)]
    disable_enforce_layers: bool,

    /// Globally disable enforce_privacy
    #[arg(long)]
    disable_enforce_privacy: bool,

    /// Globally disable enforce_visibility
    #[arg(long)]
    disable_enforce_visibility: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[clap(about = "Run check, validate, and lint")]
    All,

    #[clap(about = "Just saying hi")]
    Greet,

    #[clap(about = "Set up packs in this project")]
    Init {
        /// Generate packwerk compatible packwerk.yml instead of packs.yml
        #[arg(long)]
        use_packwerk: bool,
    },

    #[clap(about = "Create a new pack")]
    Create { name: String },

    #[clap(about = "Look for violations in the codebase")]
    Check {
        /// Ignore recorded violations when reporting violations
        #[arg(long)]
        ignore_recorded_violations: bool,

        files: Vec<String>,
    },

    #[clap(about = "Check file contents piped to stdin")]
    CheckContents {
        /// Ignore recorded violations when reporting violations
        #[arg(long)]
        ignore_recorded_violations: bool,

        file: String,
    },

    #[clap(
        about = "Update package_todo.yml files with the current violations"
    )]
    Update {
        /// Files to scope the update to (merge mode). Without files, replaces all package_todo.yml files.
        files: Vec<String>,

        /// Expand file arguments to their owning pack(s), updating all files in those packs
        #[arg(long)]
        pack: bool,

        /// Only update violations for this constant (e.g. "::Foo")
        #[arg(long)]
        constant: Option<String>,

        /// Only update violations of this type (e.g. "dependency", "privacy")
        #[arg(long)]
        violation_type: Option<String>,
    },

    #[clap(about = "Look for validation errors in the codebase")]
    Validate,

    #[clap(about = "Add a dependency from one pack to another")]
    AddDependency {
        /// The pack that depends on another pack
        from: String,

        /// The pack that is depended on
        to: String,
    },

    #[clap(
        about = "Add missing dependencies for the pack that defines the constant"
    )]
    UpdateDependenciesForConstant {
        /// Update every pack that references this constant
        constant: String,
    },

    #[clap(
        about = "Check for dependencies that when removed produce no violations."
    )]
    CheckUnnecessaryDependencies {
        #[arg(long)]
        auto_correct: bool,
    },

    #[clap(about = "Add everything a pack depends on (may cause cycles)")]
    AddDependencies { pack_name: String },

    #[clap(about = "Lint package.yml files", aliases = ["lint"])]
    LintPackageYmlFiles,

    #[clap(
        about = "Expose monkey patches of the Ruby stdlib, gems your app uses, and your application itself"
    )]
    ExposeMonkeyPatches(ExposeMonkeyPatchesArgs),

    #[clap(
        about = "`rm -rf` on your cache directory, default `tmp/cache/packwerk`"
    )]
    DeleteCache,

    #[clap(
        about = "List packs based on configuration in packwerk.yml (for debugging purposes)"
    )]
    ListPacks,

    #[clap(about = "List packs that depend on a pack")]
    ListPackDependencies {
        /// The pack that is depended on
        pack: String,
    },

    #[clap(
        about = "List analyzed files based on configuration in packwerk.yml (for debugging purposes)"
    )]
    ListIncludedFiles,

    #[clap(
        about = "List the constants that packs sees and where it sees them (for debugging purposes)"
    )]
    ListDefinitions(ListDefinitionsArgs),

    #[clap(
        about = "List constant references and their definition files (for test selection)"
    )]
    ListReferences(ListReferencesArgs),
}

#[derive(Debug, Args)]
struct ListDefinitionsArgs {
    /// Show constants with multiple definitions only
    #[arg(short, long)]
    ambiguous: bool,
}

#[derive(Debug, Args)]
struct ListReferencesArgs {
    /// Output format: 'json' or 'text'
    #[arg(short, long, default_value = "json")]
    format: String,

    /// Output file path
    #[arg(short, long)]
    out: Option<PathBuf>,
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
    fn absolute_project_root(&self) -> anyhow::Result<PathBuf> {
        self.project_root
            .canonicalize()
            .map_err(anyhow::Error::from)
    }
}

pub fn run() -> anyhow::Result<()> {
    let args = Args::parse();
    let absolute_root = args
        .absolute_project_root()
        .expect("Issue getting absolute_project_root!");

    install_logger(args.debug);

    // The `init` command is run in directories which have no configuration yet, however, below we
    // attempt to load configuration before the CLI commands are processed. To avoid this catch-22
    // we process `init` here, before configuration load. In future consider restructuring so that
    // command matching is not dependent on configuration files being available.
    if let Command::Init { use_packwerk } = args.command {
        packs::init(&absolute_root, use_packwerk)?
    }

    // Input filesize TBD
    let mut configuration = packs::configuration::get(&absolute_root, &0)?;

    if args.print_files {
        configuration.print_files = true;
    }

    if args.experimental_parser {
        debug!("Using experimental parser");
        configuration.experimental_parser = true;
    }

    if args.no_cache {
        debug!("Cache is disabled");
        configuration.cache_enabled = false;
    }

    if args.disable_enforce_dependencies {
        configuration.disable_enforce_dependencies = true;
    }

    if args.disable_enforce_folder_privacy {
        configuration.disable_enforce_folder_privacy = true;
    }

    if args.disable_enforce_layers {
        configuration.disable_enforce_layers = true;
    }

    if args.disable_enforce_privacy {
        configuration.disable_enforce_privacy = true;
    }

    if args.disable_enforce_visibility {
        configuration.disable_enforce_visibility = true;
    }

    match args.command {
        Command::All => {
            let check_result = packs::check(&configuration, vec![]);
            let validate_result = packs::validate(&configuration);
            let lint_result = packs::lint_package_yml_files(&configuration);

            check_result.and(validate_result).and(lint_result)
        }
        Command::Greet => {
            packs::greet();
            Ok(())
        }
        Command::Init { use_packwerk } => {
            println!(
                "Successfully initialized packs{} in this directory!",
                if use_packwerk { "/packwerk" } else { "" }
            );
            Ok(())
        }
        Command::ListPacks => {
            packs::list(configuration);
            Ok(())
        }
        Command::ListPackDependencies { pack } => {
            packs::list_dependencies(&configuration, pack)
        }
        Command::AddDependency { from, to } => {
            packs::add_dependency(&configuration, from, to)
        }
        Command::ListIncludedFiles => packs::list_included_files(configuration),
        Command::Check {
            ignore_recorded_violations,
            files,
        } => {
            configuration.ignore_recorded_violations =
                ignore_recorded_violations;
            configuration.input_files_count = files.len();
            packs::check(&configuration, files)
        }
        Command::CheckContents {
            ignore_recorded_violations,
            file,
        } => {
            configuration.ignore_recorded_violations =
                ignore_recorded_violations;

            let absolute_path = get_absolute_path(file.clone(), &configuration);
            configuration.stdin_file_path = Some(absolute_path);
            configuration.input_files_count = 1;
            packs::check(&configuration, vec![file])
        }
        Command::Update {
            files,
            pack,
            constant,
            violation_type,
        } => packs::update(
            &configuration,
            &packs::checker::UpdateOptions {
                files,
                expand_to_pack: pack,
                constant_name: constant,
                violation_type,
            },
        ),
        Command::Validate => {
            packs::validate(&configuration)
            // Err("💡 Please use `packs check` to detect dependency cycles and run other configuration validations".into())
        }
        Command::CheckUnnecessaryDependencies { auto_correct } => {
            packs::check_unnecessary_dependencies(&configuration, auto_correct)
        }
        Command::AddDependencies { pack_name } => {
            packs::add_dependencies(&configuration, &pack_name)
        }
        Command::UpdateDependenciesForConstant { constant } => Ok(
            packs::update_dependencies_for_constant(&configuration, &constant)?,
        ),
        Command::DeleteCache => {
            packs::delete_cache(configuration);
            Ok(())
        }
        Command::ListDefinitions(args) => {
            let ambiguous = args.ambiguous;
            packs::list_definitions(&configuration, ambiguous)
        }
        Command::ListReferences(args) => packs::list_references(
            &configuration,
            &args.format,
            args.out.as_deref(),
        ),
        Command::ExposeMonkeyPatches(args) => packs::expose_monkey_patches(
            &configuration,
            &args.rubydir,
            &args.gemdir,
        ),
        Command::LintPackageYmlFiles => {
            packs::lint_package_yml_files(&configuration)
        }
        Command::Create { name } => packs::create(&configuration, name),
    }
}
