//! Tracing setup shared by every binary.

/// Initialise the global subscriber (`RUST_LOG`, default `info`).
pub fn init() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .try_init();
}
