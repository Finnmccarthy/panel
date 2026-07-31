use crate::config::Config;
use std::{path::PathBuf, process::ExitCode};

mod build;
mod cache_key;
mod config;
mod group;
mod panel;
mod runtime;
mod server;
mod store;
mod translations;

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
