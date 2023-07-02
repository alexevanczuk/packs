use tracing::metadata::LevelFilter;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;

//
// This allows us to run the binary with timing and debug output, like so:
// $ packs --debug update
//    0.000116875s DEBUG src/packs/configuration.rs:52: Beginning to build configuration
//    0.000253917s DEBUG src/packs/walk_directory.rs:24: Beginning directory walk
//    0.014609292s DEBUG src/packs/walk_directory.rs:144: Finished directory walk
//    ...
//    0.072214542s DEBUG src/packs/checker.rs:159: Filtering out recorded violations
//    0.072355292s DEBUG src/packs/checker.rs:168: Finished filtering out recorded violations
//
pub fn install_logger(debug: bool) {
    let filter = tracing_subscriber::filter::Targets::new()
        .with_default(LevelFilter::DEBUG)
        // Disable all traces from `globset`.
        .with_target("globset", LevelFilter::OFF);

    let subscriber_builder = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_level(true)
        .with_writer(std::io::stderr)
        .with_file(true)
        .with_span_events(FmtSpan::ACTIVE)
        .with_line_number(true);

    if debug {
        // If debug mode is on, let's always show the backtrace,
        // which helps make debugging panic messages simpler.
        // There may be a more standard way to do this than setting the backtrace,
        // but it works for now.
        // Note another value instead of "1" is "FULL". For now, "1" is enough.
        std::env::set_var("RUST_BACKTRACE", "1");

        // Let's also set the log level to be debug with this flag.
        let subscriber_builder =
            subscriber_builder.with_max_level(Level::DEBUG);
        let subscriber = subscriber_builder.finish();
        let layered_subscriber = filter.with_subscriber(subscriber);
        layered_subscriber.init();
    } else {
        let subscriber = subscriber_builder.finish();
        let layered_subscriber = filter.with_subscriber(subscriber);
        layered_subscriber.init();
    }
}
