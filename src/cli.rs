use clap::{Parser, Subcommand};

use crate::commands::{
    disable, enable, import, info, init, install, list, marketplace, pack, sync, target,
    uninstall, update,
};

#[derive(Debug, Parser)]
#[command(name = "plm")]
#[command(about = "Plugin Manager CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// ターゲット環境の管理
    Target(target::Args),

    /// マーケットプレイスの管理
    Marketplace(marketplace::Args),

    /// マーケットプレイスからプラグインをインストール
    Install(install::Args),

    /// インストール済みのコンポーネント一覧
    List(list::Args),

    /// コンポーネントの詳細表示
    Info(info::Args),

    /// コンポーネントを有効化
    Enable(enable::Args),

    /// コンポーネントを無効化
    Disable(disable::Args),

    /// コンポーネントを削除
    Uninstall(uninstall::Args),

    /// コンポーネントを更新
    Update(update::Args),

    /// テンプレート生成
    Init(init::Args),

    /// 配布用パッケージ作成
    Pack(pack::Args),

    /// 環境間同期
    Sync(sync::Args),

    /// Claude Code Plugin からインポート
    Import(import::Args),
}
