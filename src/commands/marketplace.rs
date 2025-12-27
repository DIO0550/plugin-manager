use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// 登録済みマーケットプレイス一覧を表示
    List,

    /// マーケットプレイスを追加
    Add {
        /// GitHubリポジトリ (owner/repo) またはURL
        source: String,

        /// マーケットプレイス名（未指定ならリポジトリ名から自動設定）
        #[arg(long)]
        name: Option<String>,
    },

    /// マーケットプレイスを削除
    Remove {
        /// マーケットプレイス名
        name: String,
    },

    /// マーケットプレイスのキャッシュを更新
    Update {
        /// 特定のマーケットプレイスのみ更新（未指定なら全て）
        name: Option<String>,
    },
}

pub async fn run(args: Args) -> Result<(), String> {
    match args.command {
        Command::List => {
            println!("marketplace list: not implemented");
            Ok(())
        }
        Command::Add { source, name } => {
            println!("marketplace add {source} (name: {name:?}): not implemented");
            Ok(())
        }
        Command::Remove { name } => {
            println!("marketplace remove {name}: not implemented");
            Ok(())
        }
        Command::Update { name } => {
            println!("marketplace update {name:?}: not implemented");
            Ok(())
        }
    }
}
