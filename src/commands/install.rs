use crate::source::parse_source;
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
    /// owner/repo 形式、または plugin@marketplace 形式
    pub source: String,

    /// コンポーネント種別を指定（未指定なら自動検出の想定）
    #[arg(long = "type", value_enum)]
    pub component_type: Option<ComponentType>,

    /// 特定ターゲットにだけ入れる
    #[arg(long, value_enum)]
    pub target: Option<TargetKind>,

    /// personal / project
    #[arg(long)]
    pub scope: Option<String>,

    /// キャッシュを無視して再ダウンロード
    #[arg(long)]
    pub force: bool,
}

pub async fn run(args: Args) -> std::result::Result<(), String> {
    // ソースをパース（具体的な型を意識しない）
    let source = parse_source(&args.source).map_err(|e| e.to_string())?;

    // ダウンロード
    let cached_plugin = source
        .download(args.force)
        .await
        .map_err(|e| e.to_string())?;

    println!("\nPlugin downloaded successfully!");
    println!("  Name: {}", cached_plugin.name);
    println!("  Version: {}", cached_plugin.manifest.version);
    println!("  Path: {}", cached_plugin.path.display());
    println!("  Ref: {}", cached_plugin.git_ref);
    println!("  SHA: {}", cached_plugin.commit_sha);

    if let Some(desc) = &cached_plugin.manifest.description {
        println!("  Description: {}", desc);
    }

    // コンポーネント情報
    println!("\nComponents:");
    if cached_plugin.manifest.has_skills() {
        println!("  - Skills: {}", cached_plugin.manifest.skills.as_ref().unwrap());
    }
    if cached_plugin.manifest.has_agents() {
        println!("  - Agents: {}", cached_plugin.manifest.agents.as_ref().unwrap());
    }
    if cached_plugin.manifest.has_commands() {
        println!("  - Commands: {}", cached_plugin.manifest.commands.as_ref().unwrap());
    }

    // TODO: ターゲットへのデプロイ処理を実装
    println!("\nNote: Deployment to targets not yet implemented");

    Ok(())
}
