use clap::{Parser, Subcommand, ValueEnum};

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
    Target(TargetArgs),

    /// マーケットプレイスの管理
    Marketplace(MarketplaceArgs),

    /// マーケットプレイスからプラグインをインストール
    Install(InstallArgs),

    /// インストール済みのコンポーネント一覧
    List(ListArgs),

    /// コンポーネントの詳細表示
    Info(InfoArgs),

    /// コンポーネントを有効化
    Enable(EnableArgs),

    /// コンポーネントを無効化
    Disable(DisableArgs),

    /// コンポーネントを削除
    Uninstall(UninstallArgs),

    /// コンポーネントを更新
    Update(UpdateArgs),

    /// テンプレート生成
    Init(InitArgs),

    /// 配布用パッケージ作成
    Pack(PackArgs),

    /// 環境間同期
    Sync(SyncArgs),

    /// Claude Code Plugin からインポート
    Import(ImportArgs),
}

#[derive(Debug, Parser)]
pub struct TargetArgs {
    #[command(subcommand)]
    pub command: TargetCommand,
}

#[derive(Debug, Subcommand)]
pub enum TargetCommand {
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

#[derive(Debug, Parser)]
pub struct MarketplaceArgs {
    #[command(subcommand)]
    pub command: MarketplaceCommand,
}

#[derive(Debug, Subcommand)]
pub enum MarketplaceCommand {
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

#[derive(Debug, Clone, ValueEnum)]
pub enum ComponentType {
    Skill,
    Agent,
    Prompt,
    Instruction,
}

#[derive(Debug, Parser)]
pub struct InstallArgs {
    /// owner/repo 形式
    pub repo: String,

    /// コンポーネント種別を指定（未指定なら自動検出の想定）
    #[arg(long = "type", value_enum)]
    pub component_type: Option<ComponentType>,

    /// 特定ターゲットにだけ入れる
    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,

    /// personal / project
    #[arg(long)]
    pub scope: Option<String>,

    /// Claude Code Plugin 形式から抽出してインストール
    #[arg(long)]
    pub from_plugin: bool,
}

#[derive(Debug, Parser)]
pub struct ListArgs {
    #[arg(long = "type", value_enum)]
    pub component_type: Option<ComponentType>,

    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,
}

#[derive(Debug, Parser)]
pub struct InfoArgs {
    pub name: String,
}

#[derive(Debug, Parser)]
pub struct EnableArgs {
    pub name: String,

    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,
}

#[derive(Debug, Parser)]
pub struct DisableArgs {
    pub name: String,

    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,
}

#[derive(Debug, Parser)]
pub struct UninstallArgs {
    pub name: String,

    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,
}

#[derive(Debug, Parser)]
pub struct UpdateArgs {
    pub name: Option<String>,
}

#[derive(Debug, Parser)]
pub struct InitArgs {
    pub name: String,

    #[arg(long = "type", value_enum)]
    pub component_type: ComponentType,
}

#[derive(Debug, Parser)]
pub struct PackArgs {
    pub path: String,
}

#[derive(Debug, Parser)]
pub struct SyncArgs {
    #[arg(long, value_enum)]
    pub from: TargetKind,

    #[arg(long, value_enum)]
    pub to: TargetKind,

    #[arg(long = "type", value_enum)]
    pub component_type: Option<ComponentType>,
}

#[derive(Debug, Parser)]
pub struct ImportArgs {
    /// owner/repo 形式
    pub repo: String,

    #[arg(long = "type", value_enum)]
    pub component_type: Option<ComponentType>,

    #[arg(long)]
    pub component: Option<String>,
}
