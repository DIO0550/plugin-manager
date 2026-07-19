// Allow unused code for future expansion
#![allow(dead_code)]

mod application;
mod cli;
mod commands;
mod component;
mod config;
mod env;
mod error;
mod format;
mod fs;
mod hooks;
mod host;
mod http;
mod import;
mod install;
mod marketplace;
mod output;
mod parser;
mod path_ext;
mod plugin;
mod repo;
mod scan;
mod source;
mod sync;
mod target;
mod tui;

use crate::cli::Cli;
use crate::error::{ErrorFormatter, RichError};
use clap::Parser;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let verbose = cli.verbose;

    if let Err(plm_err) = commands::dispatch(cli).await {
        let rich: RichError = plm_err.into();
        let formatted = ErrorFormatter::new(verbose).format(&rich);
        eprintln!("{formatted}");
        std::process::exit(1);
    }
}
