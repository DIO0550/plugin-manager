use crate::cli::Command;

pub mod disable;
pub mod enable;
pub mod import;
pub mod info;
pub mod init;
pub mod install;
pub mod list;
pub mod managed;
pub mod marketplace;
pub mod pack;
pub mod sync;
pub mod target;
pub mod uninstall;
pub mod update;

pub async fn dispatch(cli: crate::cli::Cli) -> Result<(), String> {
    match cli.command {
        Command::Target(args) => target::run(args).await,
        Command::Install(args) => install::run(args).await,
        Command::List(args) => list::run(args).await,
        Command::Info(args) => info::run(args).await,
        Command::Enable(args) => enable::run(args).await,
        Command::Disable(args) => disable::run(args).await,
        Command::Uninstall(args) => uninstall::run(args).await,
        Command::Update(args) => update::run(args).await,
        Command::Init(args) => init::run(args).await,
        Command::Pack(args) => pack::run(args).await,
        Command::Sync(args) => sync::run(args).await,
        Command::Import(args) => import::run(args).await,
        Command::Marketplace(args) => marketplace::run(args).await,
        Command::Managed => managed::run().await,
    }
}
