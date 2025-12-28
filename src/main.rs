mod cli;
mod commands;
mod error;
mod github;
mod marketplace;
mod plugin;

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
