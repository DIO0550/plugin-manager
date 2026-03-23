//! プラグインインストールコマンド
//!
//! ## フロー
//!
//! 1. TUI選択（ダウンロード前）
//!    - `--target` 未指定時: TUIでターゲット選択
//!    - `--scope` 未指定時: TUIでスコープ選択
//! 2. ダウンロード
//! 3. 配置

use crate::component::ComponentKind;
use crate::install::{self, PlaceRequest};
use crate::output::CommandSummary;
use crate::target::{all_targets, parse_target, Scope, TargetKind};
use crate::tui;
use clap::Parser;
use std::env;

#[derive(Debug, Parser)]
pub struct Args {
    /// owner/repo 形式、または plugin@marketplace 形式
    pub source: String,

    /// コンポーネント種別を指定（複数指定可、未指定なら全コンポーネント）
    #[arg(long = "type", value_enum)]
    pub component_type: Option<Vec<ComponentKind>>,

    /// デプロイ先ターゲット（複数指定可、未指定ならTUIで選択）
    #[arg(long, value_enum)]
    pub target: Option<Vec<TargetKind>>,

    /// デプロイスコープ（未指定ならTUIで選択）
    #[arg(long, value_enum)]
    pub scope: Option<Scope>,

    /// キャッシュを無視して再ダウンロード
    #[arg(long)]
    pub force: bool,
}

pub async fn run(args: Args) -> std::result::Result<(), String> {
    // 1. ターゲット選択（ダウンロード前）
    let target_names: Vec<String> = match &args.target {
        Some(targets) => targets.iter().map(|t| t.as_str().to_string()).collect(),
        None => {
            // TUIでターゲット選択
            let available = all_targets();
            let available_refs: Vec<&dyn crate::target::Target> =
                available.iter().map(|t| t.as_ref()).collect();
            let all_components = ComponentKind::all().to_vec();

            tui::select_targets(&available_refs, &all_components).map_err(|e| e.to_string())?
        }
    };

    if target_names.is_empty() {
        return Err("No targets selected".to_string());
    }

    // 2. スコープ選択（ダウンロード前）
    let scope: Scope = match args.scope {
        Some(s) => s,
        None => {
            // TUIでスコープ選択
            tui::select_scope().map_err(|e| e.to_string())?
        }
    };

    println!("\nSelected targets: {}", target_names.join(", "));
    println!("Selected scope: {}", scope);

    // 3. ダウンロード
    println!("\nDownloading plugin...");
    let downloaded = install::download_plugin(&args.source, args.force).await?;

    println!("\nPlugin downloaded successfully!");
    println!("  Name: {}", downloaded.name());
    println!("  Version: {}", downloaded.version());
    println!("  Path: {}", downloaded.cached_path().display());
    println!("  Ref: {}", downloaded.cached_plugin().git_ref);
    println!("  SHA: {}", downloaded.cached_plugin().commit_sha);

    if let Some(desc) = downloaded.description() {
        println!("  Description: {}", desc);
    }

    // コンポーネント情報
    println!("\nComponents:");
    if let Some(skills) = downloaded.cached_plugin().skills() {
        println!("  - Skills: {}", skills);
    }
    if let Some(agents) = downloaded.cached_plugin().agents() {
        println!("  - Agents: {}", agents);
    }
    if let Some(commands) = downloaded.cached_plugin().commands() {
        println!("  - Commands: {}", commands);
    }

    // 4. コンポーネントをスキャン
    let type_filter = args.component_type.as_deref();
    let scanned = install::scan_plugin(&downloaded, type_filter)?;

    // 5. ターゲットを解決
    let targets: Vec<Box<dyn crate::target::Target>> = target_names
        .iter()
        .map(|name| parse_target(name).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    // 6. 配置
    println!("\nPlacing to targets...");

    let project_root = env::current_dir().map_err(|e| e.to_string())?;

    let result = install::place_plugin(&PlaceRequest {
        scanned: &scanned,
        targets: &targets,
        scope,
        project_root: &project_root,
    });

    // 7. 結果表示（ターゲット単位でグループ化、旧実装互換）
    println!("\nPlacement Results:");

    for target_name in &target_names {
        let mut target_success = true;

        // このターゲットの成功結果を表示
        for success in result.successes.iter().filter(|s| &s.target == target_name) {
            let suffix = match (&success.source_format, &success.dest_format) {
                (Some(src), Some(dst)) => format!(" (Converted: {} → {})", src, dst),
                _ => String::new(),
            };
            println!(
                "  + {} {}: {} -> {}{}",
                success.target,
                success.component_kind,
                success.component_name,
                success.target_path.display(),
                suffix
            );

            // Hook 変換警告を表示
            for warning in &success.hook_warnings {
                println!("    \u{26a0} {}", warning);
            }
        }

        // このターゲットの失敗結果を表示
        for failure in result.failures.iter().filter(|f| &f.target == target_name) {
            println!(
                "  x {} {}: {} - {}",
                failure.target, failure.component_kind, failure.component_name, failure.error
            );
            target_success = false;
        }

        if !target_success {
            println!("  {} - FAILED", target_name);
        }
    }

    // 結果サマリー
    let summary = CommandSummary::format(result.successes.len(), result.failures.len());
    println!("\n{} {}", summary.prefix, summary.message);

    Ok(())
}
