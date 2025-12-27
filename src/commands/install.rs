use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum ComponentType {
    Skill,
    Agent,
    Prompt,
    Instruction,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TargetKind {
    Codex,
    Copilot,
}

#[derive(Debug, Parser)]
pub struct Args {
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

pub async fn run(args: Args) -> Result<(), String> {
    println!("install: {:?}", args);
    Err("not implemented".to_string())
}
