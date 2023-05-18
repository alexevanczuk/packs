use crate::project::Project;
use clap::{Parser, Subcommand};
use core::fmt;
use error_stack::{Context, Result, ResultExt};
use path_clean::PathClean;
use std::{
    fs::File,
    path::{Path, PathBuf},
    // process,
};

mod error_stack_ext;
mod project;
use error_stack_ext::IntoContext;

#[derive(Subcommand, Debug)]
enum Command {
    /// Check for violations.
    Check,

    /// Update package_todo.yml files with new violations
    Update,

    /// Alias for check for packwerk compatibility
    Validate,
    // Future commands can be the ones in use_packs
}

/// A CLI to interact with packs
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Path for the configuration file
    #[arg(long, default_value = "packwerk.yml")] // this could later default to packs.yml, or auto-detect
    config_path: PathBuf,

    /// Path for the root of the project
    #[arg(long, default_value = ".")]
    project_root: PathBuf,
}

impl Args {
    fn absolute_project_root(&self) -> Result<PathBuf, Error> {
        self.project_root.canonicalize().into_context(Error::Io)
    }

    fn absolute_config_path(&self) -> Result<PathBuf, Error> {
        Ok(self.absolute_path(&self.config_path)?.clean())
    }

    fn absolute_path(&self, path: &Path) -> Result<PathBuf, Error> {
        Ok(self.absolute_project_root()?.join(path))
    }
}

#[derive(Debug)]
pub enum Error {
    Io,
    ValidationFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io => fmt.write_str("Error::Io"),
            Error::ValidationFailed => fmt.write_str("Error::ValidationFailed"),
        }
    }
}

impl Context for Error {}

fn main() -> Result<(), Error> {
    install_logger();
    maybe_print_errors(cli())?;

    Ok(())
}

fn cli() -> Result<(), Error> {
    let args = Args::parse();

    let config_path = args.absolute_config_path()?;
    let project_root = args.absolute_project_root()?;

    let config_file = File::open(&config_path)
        .into_context(Error::Io)
        .attach_printable(format!("Can't open config file: {}", config_path.to_string_lossy()))?;

    let config = serde_yaml::from_reader(config_file).into_context(Error::Io)?;

    // let ownership = Ownership::build(Project::build(&project_root, &codeowners_file_path, &config).change_context(Error::Io)?);
    // We could generate a view of
    match args.command {
        Command::Validate => {} // ownership.validate().into_context(Error::ValidationFailed)?,
        Command::Check => {
            // std::fs::write(codeowners_file_path, ownership.generate_file()).into_context(Error::Io)?;
        }
        Command::Update => todo!(),
    }

    Ok(())
}

fn maybe_print_errors(result: Result<(), Error>) -> Result<(), Error> {
    // if let Err(error) = result {
    //     if let Some(validation_errors) = error.downcast_ref::<ownership::ValidatorErrors>() {
    //         println!("{}", validation_errors);
    //         process::exit(-1);
    //     } else {
    //         Err(error)?
    //     }
    // }

    Ok(())
}

fn install_logger() {
    // tracing_subscriber::fmt()
    //     .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    //     .with_target(true)
    //     .with_timer(tracing_subscriber::fmt::time::uptime())
    //     .with_level(true)
    //     .with_writer(std::io::stderr)
    //     .init();
}
