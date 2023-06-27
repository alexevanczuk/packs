use packs::packs::cli;
use packs::packs::logger::install_logger;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    install_logger();
    cli::run()
}
