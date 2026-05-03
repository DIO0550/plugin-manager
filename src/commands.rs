use crate::cli::Command;

pub mod deploy;
pub mod info;
pub mod init;
pub mod lifecycle;
pub mod list;
pub mod managed;
pub mod marketplace;
pub mod pack;
pub mod target;

/// Dispatch the parsed CLI command to the matching handler.
///
/// # Arguments
///
/// * `cli` - Parsed top-level CLI invocation containing the selected subcommand.
pub async fn dispatch(cli: crate::cli::Cli) -> Result<(), String> {
    match cli.command {
        Command::Target(args) => target::run(args).await,
        Command::Install(args) => deploy::install::run(args).await,
        Command::List(args) => list::run(args).await,
        Command::Info(args) => info::run(args).await,
        Command::Enable(args) => lifecycle::enable::run(args).await,
        Command::Disable(args) => lifecycle::disable::run(args).await,
        Command::Uninstall(args) => lifecycle::uninstall::run(args).await,
        Command::Update(args) => lifecycle::update::run(args).await,
        Command::Init(args) => init::run(args).await,
        Command::Pack(args) => pack::run(args).await,
        Command::Link(args) => deploy::link::run(args).await,
        Command::Unlink(args) => deploy::unlink::run(args).await,
        Command::Sync(args) => deploy::sync::run(args).await,
        Command::Import(args) => deploy::import::run(args).await,
        Command::Marketplace(args) => marketplace::run(args).await,
        Command::Managed => managed::run().await,
    }
}
