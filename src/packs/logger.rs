//
// This allows us to run the binary with timing and debug output, like so:
// $ RUST_LOG=debug ../packs-rs/target/release/packs update
//    0.001426500s DEBUG packs::packs::configuration: src/packs/configuration.rs:263: Beginning to read configuration
//    0.843766458s DEBUG packs::packs::configuration: src/packs/configuration.rs:308: Finished reading configuration
//    0.845547750s DEBUG packs::packs::cache: src/packs/cache.rs:243: Writing cache for 29078 files
//    1.855275708s DEBUG packs::packs::cache: src/packs/cache.rs:252: Finished writing cache for 29078 files
//
pub fn install_logger() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(true)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_level(true)
        .with_writer(std::io::stderr)
        .with_file(true)
        .with_line_number(true)
        .init();
}
