use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// 現在のターゲット一覧を表示
    List,

    /// ターゲット環境を追加
    Add {
        #[arg(value_enum)]
        target: TargetKind,
    },

    /// ターゲット環境を削除
    Remove {
        #[arg(value_enum)]
        target: TargetKind,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TargetKind {
    Codex,
    Copilot,
}

pub async fn run(args: Args) -> Result<(), String> {
    match args.command {
        Command::List => {
            println!("target list: not implemented");
            Ok(())
        }
        Command::Add { target } => {
            println!("target add {target:?}: not implemented");
            Ok(())
        }
        Command::Remove { target } => {
            println!("target remove {target:?}: not implemented");
            Ok(())
        }
    }
}
