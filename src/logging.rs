use color_eyre::eyre::Result;
use tracing_subscriber::EnvFilter;

pub fn setup() -> Result<()> {
    // Get / set backtrace
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    // Install color_eyre
    color_eyre::install()?;

    // Get/set the log level
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    // Setup tracing and tracing-subscriber
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Ok(())
}
