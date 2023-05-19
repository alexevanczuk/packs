use clap::{Parser, Subcommand};
use packs::Packs;
mod packs;

#[derive(Subcommand, Debug)]
enum Command {
    Greet,
}

/// A CLI to interact with packs
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn main() {
    cli()
}

fn cli() {
    let args = Args::parse();
    match args.command {
        Command::Greet => {
            Packs::greet();
        }
    }
}
