use crate::cli::Command;

pub mod deploy;
pub mod info;
pub mod lifecycle;
pub mod list;
pub mod manage;

/// Dispatch the parsed CLI command to the matching handler.
///
/// # Arguments
///
/// * `cli` - Parsed top-level CLI invocation containing the selected subcommand.
pub async fn dispatch(cli: crate::cli::Cli) -> Result<(), String> {
    match cli.command {
        Command::Target(args) => manage::target::run(args).await,
        Command::Install(args) => deploy::install::run(args).await,
        Command::List(args) => list::run(args).await,
        Command::Info(args) => info::run(args).await,
        Command::Enable(args) => lifecycle::enable::run(args).await,
        Command::Disable(args) => lifecycle::disable::run(args).await,
        Command::Uninstall(args) => lifecycle::uninstall::run(args).await,
        Command::Update(args) => lifecycle::update::run(args).await,
        Command::Init(args) => manage::init::run(args).await,
        Command::Pack(args) => manage::pack::run(args).await,
        Command::Link(args) => deploy::link::run(args).await,
        Command::Unlink(args) => deploy::unlink::run(args).await,
        Command::Sync(args) => deploy::sync::run(args).await,
        Command::Import(args) => deploy::import::run(args).await,
        Command::Marketplace(args) => manage::marketplace::run(args).await,
        Command::Managed => manage::managed::run().await,
    }
}
