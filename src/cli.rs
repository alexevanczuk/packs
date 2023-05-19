use crate::packs::Packs;
use clap::{Parser, Subcommand};

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

pub fn cli() {
    let args = Args::parse();
    match args.command {
        Command::Greet => {
            Packs::greet();
        }
    }
}
