mod cli;
mod commands;
mod component;
mod config;
mod domain;
mod env;
mod error;
mod host;
mod http;
mod marketplace;
mod output;
mod plugin;
mod repo;
mod source;
mod target;
mod tui;

use clap::Parser;
use crate::cli::Cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(err) = commands::dispatch(cli).await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
