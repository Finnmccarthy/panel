#[cfg(unix)]
use crate::config::Config;
#[cfg(unix)]
use std::path::PathBuf;
use std::process::ExitCode;

#[cfg(unix)]
mod build;
#[cfg(unix)]
mod cache_key;
#[cfg(unix)]
mod config;
#[cfg(unix)]
mod group;
#[cfg(unix)]
mod panel;
#[cfg(unix)]
mod runtime;
#[cfg(unix)]
mod server;
#[cfg(unix)]
mod store;
#[cfg(unix)]
mod translations;

#[cfg(not(unix))]
fn main() -> ExitCode {
    eprintln!("the heavy supervisor is only supported on unix platforms");

    ExitCode::FAILURE
}

#[cfg(unix)]
#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
            "%Y-%m-%d %H:%M:%S %z".to_string(),
        ))
        .with_target(false)
        .with_max_level(if config::debug_logging(&|key| std::env::var(key).ok()) {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .init();

    let root = std::env::var("SUPERVISOR_ROOT").map_or_else(|_| PathBuf::from("/"), PathBuf::from);
    let config = Config::resolve(&root, &|key| std::env::var(key).ok());

    match runtime::run(config).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            tracing::error!("{err:#}");

            ExitCode::FAILURE
        }
    }
}
