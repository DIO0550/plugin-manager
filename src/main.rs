mod cli;
mod commands;

use clap::Parser;

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();

    if let Err(err) = commands::dispatch(cli).await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
