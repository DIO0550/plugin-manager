use crate::cli::Command;
use clap::CommandFactory;
use std::io::IsTerminal;

pub(crate) mod args;
pub mod deploy;
pub mod info;
pub mod lifecycle;
pub mod list;
pub mod manage;

/// サブコマンド省略時のデフォルトアクション。
///
/// `stdout().is_terminal()` を引数で受け取り、テスト可能にする。
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DefaultAction {
    LaunchManaged,
    PrintHelp,
}

pub(crate) fn decide_default_action(stdout_is_tty: bool) -> DefaultAction {
    if stdout_is_tty {
        DefaultAction::LaunchManaged
    } else {
        DefaultAction::PrintHelp
    }
}

/// Dispatch the parsed CLI command to the matching handler.
///
/// # Arguments
///
/// * `cli` - Parsed top-level CLI invocation containing the selected subcommand.
pub async fn dispatch(cli: crate::cli::Cli) -> Result<(), String> {
    match cli.command {
        Some(Command::Target(args)) => manage::target::run(args).await,
        Some(Command::Install(args)) => deploy::install::run(args).await,
        Some(Command::List(args)) => list::run(args).await,
        Some(Command::Info(args)) => info::run(args).await,
        Some(Command::Enable(args)) => lifecycle::enable::run(args).await,
        Some(Command::Disable(args)) => lifecycle::disable::run(args).await,
        Some(Command::Uninstall(args)) => lifecycle::uninstall::run(args).await,
        Some(Command::Update(args)) => lifecycle::update::run(args).await,
        Some(Command::Init(args)) => manage::init::run(args).await,
        Some(Command::Pack(args)) => manage::pack::run(args).await,
        Some(Command::Link(args)) => deploy::link::run(args).await,
        Some(Command::Unlink(args)) => deploy::unlink::run(args).await,
        Some(Command::Sync(args)) => deploy::sync::run(args).await,
        Some(Command::Import(args)) => deploy::import::run(args).await,
        Some(Command::Marketplace(args)) => manage::marketplace::run(args).await,
        // 明示呼び出しは従来通り（非TTYでもフォールバックしない=後方互換）
        Some(Command::Managed) => manage::managed::run().await,
        // サブコマンド省略時のみ TTY 判定でフォールバック
        None => run_default(std::io::stdout().is_terminal()).await,
    }
}

async fn run_default(stdout_is_tty: bool) -> Result<(), String> {
    match decide_default_action(stdout_is_tty) {
        DefaultAction::LaunchManaged => manage::managed::run().await,
        DefaultAction::PrintHelp => {
            crate::cli::Cli::command()
                .print_help()
                .map_err(|e| e.to_string())?;
            println!();
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "commands_test.rs"]
mod tests;
