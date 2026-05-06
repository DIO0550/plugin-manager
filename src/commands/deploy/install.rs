//! プラグインインストールコマンド
//!
//! ## フロー
//!
//! 1. TUI選択（ダウンロード前）
//!    - `--target` 未指定時: TUIでターゲット選択
//!    - `--scope` 未指定時: TUIでスコープ選択
//! 2. ダウンロード
//! 3. 配置

use crate::commands::args::{InteractiveScopeArgs, MultiTargetArgs};
use crate::component::ComponentKind;
use crate::install::format::render_hook_success;
use crate::install::{self, PlaceRequest, PlaceSuccess};
use crate::output::CommandSummary;
use crate::target::{all_targets, parse_target, Scope};
use crate::tui;
use clap::Parser;
use std::env;

/// `PlaceSuccess` を表示用の `(stdout 行, stderr ブロック群)` に変換する pure function。
///
/// stdout 1 行は `+ {target} {kind}: {name} -> {path}{suffix}` の形式。
/// suffix は次の優先順で決定する：
/// 1. Hook の場合: `render_hook_success` の `stdout_suffix`（cyan）
/// 2. それ以外で `source_format` / `dest_format` が両方 `Some`: ` (Converted: src → dst)`
/// 3. その他: 空
///
/// stderr ブロック群は Hook のみ `render_hook_success` の `stderr_blocks` を返す。
pub fn render_place_success_to_strings(success: &PlaceSuccess) -> (String, Vec<String>) {
    let rendered = render_hook_success(
        success.component_kind,
        success.hook_source_format,
        &success.hook_warnings,
        success.script_count,
        success.hook_count,
    );

    let suffix = match rendered.stdout_suffix {
        Some(s) => s,
        None => match (&success.source_format, &success.dest_format) {
            (Some(src), Some(dst)) => format!(" (Converted: {} → {})", src, dst),
            _ => String::new(),
        },
    };

    let stdout_line = format!(
        "  + {} {}: {} -> {}{}",
        success.target,
        success.component_kind,
        success.component_name,
        success.target_path.display(),
        suffix
    );

    (stdout_line, rendered.stderr_blocks)
}

#[derive(Debug, Parser)]
pub struct Args {
    /// owner/repo 形式、または plugin@marketplace 形式
    pub source: String,

    /// コンポーネント種別を指定（複数指定可、未指定なら全コンポーネント）
    #[arg(long = "type", value_enum)]
    pub component_type: Option<Vec<ComponentKind>>,

    #[command(flatten)]
    pub target: MultiTargetArgs,

    #[command(flatten)]
    pub scope: InteractiveScopeArgs,

    /// キャッシュを無視して再ダウンロード
    #[arg(long)]
    pub force: bool,
}

/// # Arguments
///
/// * `args` - Parsed CLI arguments for `plm install`.
pub async fn run(args: Args) -> std::result::Result<(), String> {
    // Target and scope selection happen before download so the user can cancel
    // without paying the download cost.
    let target_names: Vec<String> = match &args.target.target {
        Some(targets) => targets.iter().map(|t| t.as_str().to_string()).collect(),
        None => {
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

    let scope: Scope = match args.scope.scope {
        Some(s) => s,
        None => tui::select_scope().map_err(|e| e.to_string())?,
    };

    println!("\nSelected targets: {}", target_names.join(", "));
    println!("Selected scope: {}", scope);

    println!("\nDownloading plugin...");
    let package = install::download_plugin(&args.source, args.force)
        .await
        .map_err(|e| e.to_string())?;

    println!("\nPlugin downloaded successfully!");
    println!("  Name: {}", package.name());
    println!("  Version: {}", package.manifest().version);
    println!("  Path: {}", package.path().display());
    if let Some(meta) = crate::plugin::meta::load_meta(package.path()) {
        if let Some(ref git_ref) = meta.git_ref {
            println!("  Ref: {}", git_ref);
        }
        if let Some(ref commit_sha) = meta.commit_sha {
            println!("  SHA: {}", commit_sha);
        }
    }

    if let Some(ref desc) = package.manifest().description {
        println!("  Description: {}", desc);
    }

    println!("\nComponents:");
    if let Some(ref skills) = package.manifest().skills {
        println!("  - Skills: {}", skills);
    }
    if let Some(ref agents) = package.manifest().agents {
        println!("  - Agents: {}", agents);
    }
    if let Some(ref commands) = package.manifest().commands {
        println!("  - Commands: {}", commands);
    }

    let type_filter = args.component_type.as_deref();
    let scanned = install::scan_plugin(&package, type_filter)?;

    let targets: Vec<Box<dyn crate::target::Target>> = target_names
        .iter()
        .map(|name| parse_target(name).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    println!("\nPlacing to targets...");

    let project_root = env::current_dir().map_err(|e| e.to_string())?;

    let result = install::place_plugin(&PlaceRequest {
        scanned: &scanned,
        targets: &targets,
        scope,
        project_root: &project_root,
    });

    // Results are grouped by target to stay compatible with the legacy layout.
    println!("\nPlacement Results:");

    for target_name in &target_names {
        let mut target_success = true;

        for success in result.successes.iter().filter(|s| &s.target == target_name) {
            let (stdout_line, stderr_blocks) = render_place_success_to_strings(success);
            println!("{}", stdout_line);
            for block in &stderr_blocks {
                eprintln!("{}", block);
            }
        }

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

    let summary = CommandSummary::format(result.successes.len(), result.failures.len());
    println!("\n{} {}", summary.prefix, summary.message);

    Ok(())
}

#[cfg(test)]
#[path = "install_test.rs"]
mod tests;
